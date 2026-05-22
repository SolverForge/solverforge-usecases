const assert = require('node:assert/strict');
const test = require('node:test');

const { withBrowserEnv } = require('./support/load-browser-modules');
const { duplicateNameEmployees, shift } = require('./support/fixtures');

// These tests focus on the hospital-specific adaptation of the shared timeline widget.
test('employee view renders empty employee lanes and an explicit unassigned lane', async () => {
  await withBrowserEnv({}, async ({ document, importModule, window }) => {
    const { renderEmployeeView } = await importModule('static/app/models/employee-view.mjs');
    const container = document.createElement('div');

    renderEmployeeView({
      sf: window.SF,
      container,
      view: {
        id: 'by-employee',
        entityPlural: 'shifts',
        sourcePlural: 'employees',
        variableField: 'employeeIdx',
      },
      data: {
        employees: duplicateNameEmployees(),
        shifts: [shift({ employeeIdx: null })],
      },
    });

    assert.ok(container.querySelector('.sf-rail-timeline'));
    assert.equal(container.querySelectorAll('.sf-rail-timeline-row').length, 3);
    assert.ok(container.textContent.includes('Unassigned shifts'));
    assert.ok(container.textContent.includes('employee-1'));
    assert.ok(container.textContent.includes('employee-2'));
  });
});

test('location view uses the canonical detailed timeline surface', async () => {
  await withBrowserEnv({}, async ({ document, importModule, window }) => {
    const { renderLocationView } = await importModule('static/app/models/location-view.mjs');
    const container = document.createElement('div');

    renderLocationView({
      sf: window.SF,
      container,
      view: {
        id: 'by-location',
        entityPlural: 'shifts',
        sourcePlural: 'employees',
        variableField: 'employeeIdx',
      },
      data: {
        employees: duplicateNameEmployees(),
        shifts: [shift({ employeeIdx: 0 })],
      },
    });

    const block = container.querySelector('.sf-rail-timeline-item');
    const row = container.querySelector('.sf-rail-timeline-row');
    assert.ok(block);
    assert.ok(row);
    assert.equal(row.dataset.mode, 'detailed');
    assert.ok(!container.textContent.includes('All shifts are assigned in the current snapshot.'));
  });
});

test('timeline render does not assign read-only layout geometry', async () => {
  await withBrowserEnv({}, async ({ document, importModule, window }) => {
    const { renderLocationView } = await importModule('static/app/models/location-view.mjs');
    const geometry = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(document.createElement('div')), 'clientWidth');
    const container = document.createElement('div');

    assert.equal(typeof geometry.get, 'function');
    assert.equal(geometry.set, undefined);
    renderLocationView({
      sf: window.SF,
      container,
      view: {
        id: 'by-location',
        entityPlural: 'shifts',
        sourcePlural: 'employees',
        variableField: 'employeeIdx',
      },
      data: {
        employees: duplicateNameEmployees(),
        shifts: [shift({ employeeIdx: 0 })],
      },
    });

    assert.ok(container.querySelector('.sf-rail-timeline'));
  });
});

test('location view renders individual assignments without overview summaries', async () => {
  await withBrowserEnv({}, async ({ document, importModule, window }) => {
    const { renderLocationView } = await importModule('static/app/models/location-view.mjs');
    const container = document.createElement('div');

    renderLocationView({
      sf: window.SF,
      container,
      view: {
        id: 'by-location',
        entityPlural: 'shifts',
        sourcePlural: 'employees',
        variableField: 'employeeIdx',
      },
      data: {
        employees: duplicateNameEmployees(),
        shifts: [
          shift({ id: 'shift-1', employeeIdx: 0, start: '2024-01-01T08:00:00', end: '2024-01-01T12:00:00' }),
          shift({ id: 'shift-2', employeeIdx: 1, start: '2024-01-01T09:00:00', end: '2024-01-01T17:00:00' }),
          shift({ id: 'shift-3', employeeIdx: null, start: '2024-01-01T10:00:00', end: '2024-01-01T18:00:00' }),
        ],
      },
    });

    const detailBlocks = container.querySelectorAll('.sf-rail-timeline-item--detail');
    assert.equal(detailBlocks.length, 3);
    assert.equal(container.querySelectorAll('.sf-rail-timeline-summary-pill').length, 0);
    assert.equal(Array.from(container.querySelectorAll('.sf-block-label'))
      .filter((node) => node.textContent.trim() === 'Alex').length, 2);
    assert.ok(container.textContent.includes('Unassigned'));
  });
});

test('timeline header drag scrolls the schedule viewport horizontally', async () => {
  await withBrowserEnv({}, async ({ document, importModule, window }) => {
    const { renderLocationView } = await importModule('static/app/models/location-view.mjs');
    const container = document.createElement('div');

    renderLocationView({
      sf: window.SF,
      container,
      view: {
        id: 'by-location',
        entityPlural: 'shifts',
        sourcePlural: 'employees',
        variableField: 'employeeIdx',
      },
      data: {
        employees: duplicateNameEmployees(),
        shifts: [
          shift({ id: 'shift-1', employeeIdx: 0, start: '2024-01-01T08:00:00', end: '2024-01-01T16:00:00' }),
          shift({ id: 'shift-2', employeeIdx: 1, start: '2024-01-20T08:00:00', end: '2024-01-20T16:00:00' }),
        ],
      },
    });

    const headerViewport = container.querySelector('.sf-rail-timeline-header-viewport');
    const bodyViewport = container.querySelector('.sf-rail-timeline-body-viewport');
    assert.ok(headerViewport);
    assert.ok(bodyViewport);

    bodyViewport.scrollLeft = 120;
    headerViewport.scrollLeft = 120;
    headerViewport.eventListeners.mousedown[0]({
      button: 0,
      clientX: 200,
      preventDefault() {},
    });
    headerViewport.eventListeners.mousemove[0]({
      clientX: 150,
      preventDefault() {},
    });

    assert.equal(bodyViewport.scrollLeft, 170);
    headerViewport.eventListeners.mouseup[0]({});
  });
});

test('detailed timeline items use packed positioning from the shared timeline surface', async () => {
  await withBrowserEnv({}, async ({ document, importModule, window }) => {
    const { renderEmployeeView } = await importModule('static/app/models/employee-view.mjs');
    const container = document.createElement('div');

    renderEmployeeView({
      sf: window.SF,
      container,
      view: {
        id: 'by-employee',
        entityPlural: 'shifts',
        sourcePlural: 'employees',
        variableField: 'employeeIdx',
      },
      data: {
        employees: duplicateNameEmployees(),
        shifts: [shift({ employeeIdx: 0 })],
      },
    });

    const block = container.querySelector('.sf-rail-timeline-item--detail');
    assert.ok(block);
    assert.ok(String(block.style.top || '').length > 0);
    assert.equal(block.style.bottom, 'auto');
  });
});

test('employee view renders availability overlays from employee day preferences', async () => {
  await withBrowserEnv({}, async ({ document, importModule, window }) => {
    const { renderEmployeeView } = await importModule('static/app/models/employee-view.mjs');
    const container = document.createElement('div');

    renderEmployeeView({
      sf: window.SF,
      container,
      view: {
        id: 'by-employee',
        entityPlural: 'shifts',
        sourcePlural: 'employees',
        variableField: 'employeeIdx',
      },
      data: {
        employees: [
          { id: 'employee-1', name: 'Alex', unavailableDates: ['2024-01-01'], undesiredDates: ['2024-01-02'], desiredDates: ['2024-01-03'] },
        ],
        shifts: [shift({ employeeIdx: 0, start: '2024-01-01T08:00:00', end: '2024-01-03T16:00:00' })],
      },
    });

    assert.equal(container.querySelectorAll('.sf-rail-timeline-overlay').length, 3);
  });
});
