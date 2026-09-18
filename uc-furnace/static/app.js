import { startApp } from './app/runtime.js';

startApp().catch(function (error) {
  console.error('App bootstrap failed:', error);
});
