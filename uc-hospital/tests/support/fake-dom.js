// Extremely small DOM implementation used by browserless frontend tests.
const {
  resolveInlineSize,
  resolveBlockSize,
  resolveChildScrollWidth,
  resolveChildScrollHeight,
} = require('./fake-dom-layout');

class FakeNode {}

// Text node with just enough surface for the app and tests.
class FakeTextNode extends FakeNode {
  constructor(text) {
    super();
    this.parentNode = null;
    this.ownerDocument = null;
    this.nodeType = 3;
    this._text = String(text);
  }

  get textContent() {
    return this._text;
  }

  set textContent(value) {
    this._text = String(value);
  }
}

// Minimal `classList` implementation for the fake elements below.
class FakeClassList {
  constructor(owner) {
    this.owner = owner;
    this.values = new Set();
  }

  add(...tokens) {
    tokens.forEach((token) => {
      if (token) this.values.add(token);
    });
    this.owner._syncClassName();
  }

  remove(...tokens) {
    tokens.forEach((token) => this.values.delete(token));
    this.owner._syncClassName();
  }

  toggle(token, force) {
    if (!token) return false;
    if (force === true) {
      this.values.add(token);
      this.owner._syncClassName();
      return true;
    }
    if (force === false) {
      this.values.delete(token);
      this.owner._syncClassName();
      return false;
    }
    if (this.values.has(token)) {
      this.values.delete(token);
      this.owner._syncClassName();
      return false;
    }
    this.values.add(token);
    this.owner._syncClassName();
    return true;
  }

  contains(token) {
    return this.values.has(token);
  }

  setFromString(value) {
    this.values = new Set(String(value || '').split(/\s+/).filter(Boolean));
    this.owner._syncClassName();
  }
}

// Element implementation covering the DOM features exercised by the app.
class FakeElement extends FakeNode {
  constructor(tagName, namespaceURI) {
    super();
    this.tagName = String(tagName).toUpperCase();
    this.namespaceURI = namespaceURI || null;
    this.childNodes = [];
    this.parentNode = null;
    this.ownerDocument = null;
    this.attributes = {};
    this.style = {};
    this.dataset = {};
    this.eventListeners = {};
    this.classList = new FakeClassList(this);
    this._className = '';
    this._id = '';
    this._innerHTML = '';
    this.scrollLeft = 0;
    this.scrollTop = 0;
  }

  _syncClassName() {
    this._className = Array.from(this.classList.values).join(' ');
  }

  get className() {
    return this._className;
  }

  set className(value) {
    this.classList.setFromString(value);
  }

  get id() {
    return this._id;
  }

  set id(value) {
    this._id = String(value);
    this.attributes.id = this._id;
  }

  get innerHTML() {
    return this._innerHTML;
  }

  set innerHTML(value) {
    this._innerHTML = String(value);
    this.childNodes = [];
  }

  get children() {
    return this.childNodes.filter((child) => child instanceof FakeElement);
  }

  get clientWidth() {
    return resolveInlineSize(this);
  }

  get clientHeight() {
    return resolveBlockSize(this);
  }

  get offsetWidth() {
    return this.clientWidth;
  }

  get offsetHeight() {
    return this.clientHeight;
  }

  get scrollWidth() {
    return Math.max(resolveInlineSize(this), resolveChildScrollWidth(this));
  }

  get scrollHeight() {
    return Math.max(resolveBlockSize(this), resolveChildScrollHeight(this));
  }

  get textContent() {
    if (this.childNodes.length === 0) return '';
    return this.childNodes.map((child) => child.textContent).join('');
  }

  set textContent(value) {
    this._innerHTML = '';
    this.childNodes = [new FakeTextNode(value)];
    this.childNodes[0].parentNode = this;
    this.childNodes[0].ownerDocument = this.ownerDocument;
  }

  appendChild(child) {
    if (child == null) return child;
    child.parentNode = this;
    child.ownerDocument = this.ownerDocument;
    this.childNodes.push(child);
    this._innerHTML = '';
    return child;
  }

  removeChild(child) {
    this.childNodes = this.childNodes.filter((candidate) => candidate !== child);
    child.parentNode = null;
    return child;
  }

  setAttribute(name, value) {
    if (name === 'class') {
      this.className = value;
      return;
    }
    if (name === 'id') {
      this.id = value;
      return;
    }
    this.attributes[name] = String(value);
  }

  addEventListener(type, handler) {
    if (!this.eventListeners[type]) this.eventListeners[type] = [];
    this.eventListeners[type].push(handler);
  }

  getBoundingClientRect() {
    const width = this.clientWidth;
    const height = this.clientHeight;
    return {
      bottom: height,
      height,
      left: 0,
      right: width,
      top: 0,
      width,
      x: 0,
      y: 0,
    };
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] || null;
  }

  querySelectorAll(selector) {
    var matches = [];
    walk(this, function (node) {
      if (node instanceof FakeElement && matchesSelector(node, selector)) matches.push(node);
    });
    return matches;
  }
}

// Minimal document implementation rooted at one fake `<body>`.
class FakeDocument {
  constructor() {
    this.body = this.createElement('body');
  }

  createElement(tagName) {
    const element = new FakeElement(tagName);
    element.ownerDocument = this;
    return element;
  }

  createElementNS(namespaceURI, tagName) {
    const element = new FakeElement(tagName, namespaceURI);
    element.ownerDocument = this;
    return element;
  }

  createTextNode(text) {
    const textNode = new FakeTextNode(text);
    textNode.ownerDocument = this;
    return textNode;
  }

  getElementById(id) {
    return this.body.querySelector('#' + id);
  }

  querySelector(selector) {
    return this.body.querySelector(selector);
  }
}

// Depth-first traversal used by `querySelectorAll`.
function walk(node, visit) {
  node.childNodes.forEach((child) => {
    visit(child);
    if (child instanceof FakeElement) walk(child, visit);
  });
}

// Very small selector matcher covering the selectors used in tests.
function matchesSelector(node, selector) {
  if (selector.startsWith('.')) return node.classList.contains(selector.slice(1));
  if (selector.startsWith('#')) return node.id === selector.slice(1);
  return node.tagName.toLowerCase() === selector.toLowerCase();
}

// Factory used by the frontend test harness.
function createDom() {
  const document = new FakeDocument();
  return { document, window: { document }, Node: FakeNode };
}

module.exports = { createDom };
