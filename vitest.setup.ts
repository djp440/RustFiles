import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}

window.ResizeObserver = ResizeObserverMock as unknown as typeof ResizeObserver;

vi.spyOn(Element.prototype, 'clientHeight', 'get')
  .mockReturnValue(800);

vi.spyOn(Element.prototype, 'clientWidth', 'get')
  .mockReturnValue(1024);
