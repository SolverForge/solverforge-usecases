(function (global) {
  'use strict';

  var Fleet = global.Fleet = global.Fleet || {};

  Fleet.clonePlan = function (plan) {
    return JSON.parse(JSON.stringify(plan));
  };

  Fleet.replace = function (target) {
    target.textContent = '';
    Array.prototype.slice.call(arguments, 1).forEach(function (child) {
      if (child) target.appendChild(child);
    });
  };

  Fleet.labelize = function (value) {
    return String(value == null ? '' : value)
      .replace(/_/g, ' ')
      .replace(/\b\w/g, function (letter) { return letter.toUpperCase(); });
  };

  Fleet.constraintType = function (name) {
    return String(name).indexOf('maximize_') === 0 || String(name).indexOf('minimize_') === 0
      ? 'soft'
      : 'hard';
  };

  Fleet.findHeaderButton = function (header, label) {
    return Array.prototype.find.call(header.querySelectorAll('button'), function (button) {
      return button.textContent.trim() === label;
    });
  };

  Fleet.formatNumber = function (value) {
    return Number(value || 0).toLocaleString('en-US');
  };

  Fleet.formatPercent = function (value) {
    return Math.round(Number(value || 0) * 10) / 10 + '%';
  };
})(window);
