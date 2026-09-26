// Isometric warehouse view for order picking.
//
// This is a port of the source application's canvas renderer
// (static/logisim-view.js): the same 30-degree projection, 5x3 shelving grid,
// aisle routing, animated trolleys, and numbered pickup markers. The only
// change is the data contract. The source read nested `step.orderItem.product`
// records; SolverForge serializes flat `pickSteps` with the product fields
// already on the step, so a resolved route step carries `location` directly.

const ISO = {
  ANGLE: Math.PI / 6,

  TILE_WIDTH: 48,
  TILE_HEIGHT: 24,

  SHELF_WIDTH: 3,
  SHELF_DEPTH: 1,
  SHELF_HEIGHT: 2,

  COLS: 5,
  ROWS: 3,

  AISLE_WIDTH: 2,

  COLORS: {
    floor: '#e2e8f0',
    floorGrid: '#cbd5e1',
    shelfTop: '#ffffff',
    shelfFront: '#f1f5f9',
    shelfSide: '#e2e8f0',
    shelfBorder: '#94a3b8',
    shadow: 'rgba(0, 0, 0, 0.1)',
    path: 'rgba(59, 130, 246, 0.3)',
    pathActive: 'rgba(59, 130, 246, 0.6)',
  },
};

const COLUMNS = ['A', 'B', 'C', 'D', 'E'];
const ROWS = ['1', '2', '3'];

/**
 * Creates the warehouse canvas plus its trolley legend overlay.
 *
 * Returns an element to mount and a small controller. `render(model)` accepts
 * the view model from `orders-model.mjs`; `setSolving` starts and stops the
 * trolley animation exactly as the source did while a solve was running.
 */
export function createWarehouse() {
  const el = document.createElement('div');
  el.className = 'orders-warehouse';

  const canvas = document.createElement('canvas');
  canvas.className = 'orders-warehouse__canvas';

  const legend = document.createElement('div');
  legend.className = 'orders-legend';
  legend.appendChild(el2('div', { className: 'orders-legend__header' }, icon('fa-dolly'), text('Trolleys')));
  const legendList = el2('div', { className: 'orders-legend__list' });
  legend.appendChild(legendList);

  el.append(canvas, legend);

  const ctx = canvas.getContext('2d');
  const state = {
    canvas,
    ctx,
    dpr: 1,
    width: 0,
    height: 0,
    originX: 0,
    originY: 0,
    gridOffsetX: 0,
    gridOffsetY: 0,
    animations: new Map(),
    model: null,
    isSolving: false,
    animationId: null,
  };

  function resize() {
    const dpr = window.devicePixelRatio || 1;
    state.dpr = dpr;

    const totalWidth = (ISO.COLS * (ISO.SHELF_WIDTH + ISO.AISLE_WIDTH) + ISO.AISLE_WIDTH) * ISO.TILE_WIDTH;
    const totalHeight = (ISO.ROWS * (ISO.SHELF_DEPTH + ISO.AISLE_WIDTH) + ISO.AISLE_WIDTH) * ISO.TILE_WIDTH;

    state.width = totalWidth + 200;
    state.height = totalHeight / 2 + 300;

    canvas.width = state.width * dpr;
    canvas.height = state.height * dpr;
    canvas.style.width = `${state.width}px`;
    canvas.style.height = `${state.height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const gridSize = ISO.COLS * (ISO.SHELF_WIDTH + ISO.AISLE_WIDTH) + ISO.AISLE_WIDTH;
    const gridDepth = ISO.ROWS * (ISO.SHELF_DEPTH + ISO.AISLE_WIDTH) + ISO.AISLE_WIDTH;
    state.gridOffsetX = gridSize / 2;
    state.gridOffsetY = gridDepth / 2;
    state.originX = state.width / 2;
    state.originY = state.height / 2 - 50;
  }

  function toScreen(x, y, z = 0) {
    const centeredX = x - state.gridOffsetX;
    const centeredY = y - state.gridOffsetY;
    return {
      x: state.originX + (centeredX - centeredY) * (ISO.TILE_WIDTH / 2),
      y: state.originY + (centeredX + centeredY) * (ISO.TILE_HEIGHT / 2) - z * ISO.TILE_HEIGHT,
    };
  }

  function drawFloor() {
    const gridSize = ISO.COLS * (ISO.SHELF_WIDTH + ISO.AISLE_WIDTH) + ISO.AISLE_WIDTH;
    const gridDepth = ISO.ROWS * (ISO.SHELF_DEPTH + ISO.AISLE_WIDTH) + ISO.AISLE_WIDTH;
    for (let x = 0; x < gridSize; x++) {
      for (let y = 0; y < gridDepth; y++) {
        const p1 = toScreen(x, y);
        const p2 = toScreen(x + 1, y);
        const p3 = toScreen(x + 1, y + 1);
        const p4 = toScreen(x, y + 1);
        ctx.beginPath();
        ctx.moveTo(p1.x, p1.y);
        ctx.lineTo(p2.x, p2.y);
        ctx.lineTo(p3.x, p3.y);
        ctx.lineTo(p4.x, p4.y);
        ctx.closePath();
        ctx.fillStyle = ISO.COLORS.floor;
        ctx.fill();
        ctx.strokeStyle = ISO.COLORS.floorGrid;
        ctx.lineWidth = 0.5;
        ctx.stroke();
      }
    }
  }

  function drawShelf(col, row, label) {
    const gridX = ISO.AISLE_WIDTH + col * (ISO.SHELF_WIDTH + ISO.AISLE_WIDTH);
    const gridY = ISO.AISLE_WIDTH + row * (ISO.SHELF_DEPTH + ISO.AISLE_WIDTH);
    const w = ISO.SHELF_WIDTH;
    const d = ISO.SHELF_DEPTH;
    const h = ISO.SHELF_HEIGHT;

    const topFront = [
      toScreen(gridX, gridY + d, h),
      toScreen(gridX + w, gridY + d, h),
      toScreen(gridX + w, gridY, h),
      toScreen(gridX, gridY, h),
    ];
    const bottomFront = [toScreen(gridX, gridY + d, 0), toScreen(gridX + w, gridY + d, 0)];
    const bottomSide = [toScreen(gridX + w, gridY, 0)];

    ctx.beginPath();
    const shadowOffset = 0.3;
    const s1 = toScreen(gridX + shadowOffset, gridY + d + shadowOffset, 0);
    const s2 = toScreen(gridX + w + shadowOffset, gridY + d + shadowOffset, 0);
    const s3 = toScreen(gridX + w + shadowOffset, gridY + shadowOffset, 0);
    const s4 = toScreen(gridX + shadowOffset, gridY + shadowOffset, 0);
    ctx.moveTo(s1.x, s1.y);
    ctx.lineTo(s2.x, s2.y);
    ctx.lineTo(s3.x, s3.y);
    ctx.lineTo(s4.x, s4.y);
    ctx.closePath();
    ctx.fillStyle = ISO.COLORS.shadow;
    ctx.fill();

    ctx.beginPath();
    ctx.moveTo(topFront[0].x, topFront[0].y);
    ctx.lineTo(topFront[1].x, topFront[1].y);
    ctx.lineTo(bottomFront[1].x, bottomFront[1].y);
    ctx.lineTo(bottomFront[0].x, bottomFront[0].y);
    ctx.closePath();
    ctx.fillStyle = ISO.COLORS.shelfFront;
    ctx.fill();
    ctx.strokeStyle = ISO.COLORS.shelfBorder;
    ctx.lineWidth = 1;
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(topFront[1].x, topFront[1].y);
    ctx.lineTo(topFront[2].x, topFront[2].y);
    ctx.lineTo(bottomSide[0].x, bottomSide[0].y);
    ctx.lineTo(bottomFront[1].x, bottomFront[1].y);
    ctx.closePath();
    ctx.fillStyle = ISO.COLORS.shelfSide;
    ctx.fill();
    ctx.strokeStyle = ISO.COLORS.shelfBorder;
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(topFront[0].x, topFront[0].y);
    ctx.lineTo(topFront[1].x, topFront[1].y);
    ctx.lineTo(topFront[2].x, topFront[2].y);
    ctx.lineTo(topFront[3].x, topFront[3].y);
    ctx.closePath();
    ctx.fillStyle = ISO.COLORS.shelfTop;
    ctx.fill();
    ctx.strokeStyle = ISO.COLORS.shelfBorder;
    ctx.stroke();

    const centerX = (topFront[0].x + topFront[1].x + topFront[2].x + topFront[3].x) / 4;
    const centerY = (topFront[0].y + topFront[1].y + topFront[2].y + topFront[3].y) / 4;
    ctx.font = 'bold 14px -apple-system, BlinkMacSystemFont, sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillStyle = '#475569';
    ctx.fillText(label, centerX, centerY);
  }

  function drawShelves() {
    for (let col = 0; col < ISO.COLS; col++) {
      for (let row = 0; row < ISO.ROWS; row++) {
        drawShelf(col, row, `${COLUMNS[col]},${ROWS[row]}`);
      }
    }
  }

  function drawTrolley(x, y, color, trolleyId) {
    const pos = toScreen(x, y, 0.3);
    const bodyWidth = 20;
    const bodyHeight = 14;
    const bodyDepth = 8;

    ctx.save();
    ctx.translate(pos.x, pos.y);

    ctx.beginPath();
    ctx.ellipse(0, 6, 12, 6, 0, 0, Math.PI * 2);
    ctx.fillStyle = 'rgba(0, 0, 0, 0.15)';
    ctx.fill();

    ctx.beginPath();
    ctx.moveTo(-bodyWidth / 2, -bodyDepth);
    ctx.lineTo(bodyWidth / 2, -bodyDepth);
    ctx.lineTo(bodyWidth / 2, bodyHeight - bodyDepth);
    ctx.lineTo(-bodyWidth / 2, bodyHeight - bodyDepth);
    ctx.closePath();
    ctx.fillStyle = color;
    ctx.fill();
    ctx.strokeStyle = 'rgba(0, 0, 0, 0.2)';
    ctx.lineWidth = 1;
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(-bodyWidth / 2, -bodyDepth);
    ctx.lineTo(0, -bodyDepth - 6);
    ctx.lineTo(bodyWidth / 2, -bodyDepth);
    ctx.lineTo(0, -bodyDepth + 3);
    ctx.closePath();
    ctx.fillStyle = lighten(color, 20);
    ctx.fill();
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(-bodyWidth / 2 + 3, -bodyDepth - 2);
    ctx.lineTo(-bodyWidth / 2 + 3, -bodyDepth - 12);
    ctx.lineTo(bodyWidth / 2 - 3, -bodyDepth - 12);
    ctx.lineTo(bodyWidth / 2 - 3, -bodyDepth - 2);
    ctx.strokeStyle = darken(color, 20);
    ctx.lineWidth = 2;
    ctx.stroke();

    ctx.beginPath();
    ctx.arc(0, -bodyDepth - 16, 10, 0, Math.PI * 2);
    ctx.fillStyle = 'white';
    ctx.fill();
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.stroke();

    ctx.font = 'bold 11px -apple-system, sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillStyle = color;
    ctx.fillText(trolleyId, 0, -bodyDepth - 16);

    ctx.restore();
  }

  function locationToGrid(location) {
    if (!location) return { x: 1, y: 1 };
    const match = /\(([A-E]),\s*(\d)\)/.exec(location.shelvingId || '');
    let col = 0;
    let row = 0;
    if (match) {
      col = COLUMNS.indexOf(match[1]);
      row = Number.parseInt(match[2], 10) - 1;
    }
    const shelfGridX = ISO.AISLE_WIDTH + col * (ISO.SHELF_WIDTH + ISO.AISLE_WIDTH);
    const shelfGridY = ISO.AISLE_WIDTH + row * (ISO.SHELF_DEPTH + ISO.AISLE_WIDTH);
    const aisleY = shelfGridY + ISO.SHELF_DEPTH + 0.5;
    const gridX = location.side === 'LEFT'
      ? shelfGridX - 0.5
      : shelfGridX + ISO.SHELF_WIDTH + 0.5;
    return { x: gridX, y: aisleY };
  }

  function mainAisleY() {
    return ISO.ROWS * (ISO.SHELF_DEPTH + ISO.AISLE_WIDTH) + ISO.AISLE_WIDTH + 0.5;
  }

  function buildAislePath(start, end) {
    const path = [start];
    const dx = Math.abs(start.x - end.x);
    const dy = Math.abs(start.y - end.y);
    if (dx < 2 && dy < 2) {
      path.push(end);
      return path;
    }

    const aisleY = mainAisleY();
    const startAisleY = Math.max(start.y, aisleY - 1);
    const endAisleY = Math.max(end.y, aisleY - 1);

    if (Math.abs(start.y - startAisleY) > 0.5) {
      path.push({ x: start.x, y: startAisleY });
    }
    if (Math.abs(start.x - end.x) > 0.5) {
      path.push({ x: end.x, y: startAisleY });
    }
    if (Math.abs(path[path.length - 1].y - end.y) > 0.5) {
      path.push({ x: end.x, y: end.y });
    }

    const last = path[path.length - 1];
    if (Math.abs(last.x - end.x) > 0.1 || Math.abs(last.y - end.y) > 0.1) {
      path.push(end);
    }
    return path;
  }

  function buildTrolleyPath(trolley) {
    const waypoints = [];
    const waypointTypes = [];

    if (trolley.location) {
      waypoints.push(locationToGrid(trolley.location));
      waypointTypes.push('start');
    }
    for (const step of trolley.steps) {
      if (step.location) {
        waypoints.push(locationToGrid(step.location));
        waypointTypes.push('pickup');
      }
    }
    if (waypoints.length > 1 && trolley.location) {
      waypoints.push(locationToGrid(trolley.location));
      waypointTypes.push('end');
    }
    if (waypoints.length <= 1) return { path: waypoints, pickupIndices: [] };

    const fullPath = [waypoints[0]];
    const pickupIndices = [];
    for (let i = 1; i < waypoints.length; i++) {
      const segment = buildAislePath(waypoints[i - 1], waypoints[i]);
      for (let j = 1; j < segment.length; j++) fullPath.push(segment[j]);
      if (waypointTypes[i] === 'pickup') pickupIndices.push(fullPath.length - 1);
    }
    return { path: fullPath, pickupIndices };
  }

  function drawPath(path, color, pickupIndices, active) {
    if (path.length < 2) return;
    ctx.beginPath();
    const start = toScreen(path[0].x, path[0].y, 0.1);
    ctx.moveTo(start.x, start.y);
    for (let i = 1; i < path.length; i++) {
      const point = toScreen(path[i].x, path[i].y, 0.1);
      ctx.lineTo(point.x, point.y);
    }
    ctx.strokeStyle = active ? ISO.COLORS.pathActive : ISO.COLORS.path;
    ctx.lineWidth = active ? 4 : 2;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.stroke();

    let pickupNum = 1;
    for (const idx of pickupIndices) {
      if (idx < 0 || idx >= path.length) continue;
      const point = toScreen(path[idx].x, path[idx].y, 0.5);
      ctx.beginPath();
      ctx.arc(point.x, point.y, 8, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
      ctx.strokeStyle = 'white';
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.font = 'bold 9px -apple-system, sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillStyle = 'white';
      ctx.fillText(String(pickupNum), point.x, point.y);
      pickupNum++;
    }
  }

  function positionOnPath(path, progress) {
    if (!path || path.length === 0) return { x: 0, y: 0 };
    if (path.length === 1) return path[0];
    const totalSegments = path.length - 1;
    const segmentProgress = progress * totalSegments;
    const current = Math.min(Math.floor(segmentProgress), totalSegments - 1);
    const t = segmentProgress - current;
    const start = path[current];
    const end = path[current + 1];
    if (!start || !end) return path[0] || { x: 0, y: 0 };
    return { x: start.x + (end.x - start.x) * t, y: start.y + (end.y - start.y) * t };
  }

  function render() {
    const model = state.model;
    ctx.clearRect(0, 0, state.width, state.height);
    drawFloor();
    drawShelves();
    if (!model || !model.trolleys) return;

    for (const trolley of model.trolleys) {
      const pathData = buildTrolleyPath(trolley);
      const { path, pickupIndices } = pathData;

      if (path.length > 1) drawPath(path, trolley.color, pickupIndices, state.isSolving);

      const anim = state.animations.get(trolley.id);
      let pos;
      if (anim && state.isSolving && path.length > 1) {
        const elapsed = Date.now() - anim.startTime;
        const progress = (elapsed % anim.duration) / anim.duration;
        pos = positionOnPath(path, progress);
      } else if (path.length > 0) {
        pos = path[0];
      } else {
        pos = locationToGrid(trolley.location);
      }
      if (pos) drawTrolley(pos.x, pos.y, trolley.color, trolley.id);
    }
  }

  function animate() {
    if (!state.isSolving) {
      state.animationId = null;
      return;
    }
    render();
    state.animationId = requestAnimationFrame(animate);
  }

  function syncAnimations() {
    const model = state.model;
    state.animations.clear();
    if (!model || !model.trolleys) return;
    for (const trolley of model.trolleys) {
      const path = buildTrolleyPath(trolley).path;
      state.animations.set(trolley.id, {
        startTime: Date.now() + Number.parseInt(trolley.id, 10) * 200,
        duration: Math.max(3000, path.length * 400),
        path,
      });
    }
  }

  function controllerRender(model) {
    state.model = model;
    render();
  }

  function setSolving(solving) {
    const next = !!solving;
    if (next === state.isSolving) return;
    state.isSolving = next;
    if (next) {
      syncAnimations();
      if (!state.animationId) animate();
    } else if (state.animationId) {
      cancelAnimationFrame(state.animationId);
      state.animationId = null;
      render();
    }
  }

  function renderLegend(model) {
    legendList.textContent = '';
    for (const trolley of (model && model.trolleys) || []) {
      legendList.appendChild(
        el2('div', { className: 'orders-legend__item' },
          el2('span', { className: 'orders-legend__swatch', style: { background: trolley.color } }),
          el2('span', { className: 'orders-legend__label' }, `Trolley ${trolley.id}`),
          el2('span', { className: 'orders-legend__meta' }, `${trolley.steps.length} items`)),
      );
    }
  }

  function onResize() {
    resize();
    render();
  }

  resize();
  window.addEventListener('resize', onResize);

  return {
    el,
    render(model) {
      controllerRender(model);
      renderLegend(model);
    },
    setSolving,
    destroy() {
      window.removeEventListener('resize', onResize);
      if (state.animationId) cancelAnimationFrame(state.animationId);
      state.animationId = null;
    },
  };
}

function el2(tag, attrs = {}, ...children) {
  const node = document.createElement(tag);
  if (attrs.className) node.className = attrs.className;
  if (attrs.style) Object.assign(node.style, attrs.style);
  for (const child of children) {
    if (child == null) continue;
    node.append(child);
  }
  return node;
}

function text(value) {
  return document.createTextNode(value);
}

function icon(name) {
  const i = document.createElement('i');
  i.className = `fa-solid ${name}`;
  return i;
}

function lighten(hex, percent) {
  const num = Number.parseInt(hex.slice(1), 16);
  const amt = Math.round(2.55 * percent);
  const r = Math.min(255, (num >> 16) + amt);
  const g = Math.min(255, ((num >> 8) & 0x00ff) + amt);
  const b = Math.min(255, (num & 0x0000ff) + amt);
  return `#${(0x1000000 + r * 0x10000 + g * 0x100 + b).toString(16).slice(1)}`;
}

function darken(hex, percent) {
  const num = Number.parseInt(hex.slice(1), 16);
  const amt = Math.round(2.55 * percent);
  const r = Math.max(0, (num >> 16) - amt);
  const g = Math.max(0, ((num >> 8) & 0x00ff) - amt);
  const b = Math.max(0, (num & 0x0000ff) - amt);
  return `#${(0x1000000 + r * 0x10000 + g * 0x100 + b).toString(16).slice(1)}`;
}
