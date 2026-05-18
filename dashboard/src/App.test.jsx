import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, beforeEach } from 'vitest';
import App from './App';

describe('App Component', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
    // Mock window.matchMedia
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(), // Deprecated
        removeListener: vi.fn(), // Deprecated
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
  });

  it('renders the header correctly', () => {
    render(<App />);
    expect(screen.getByText('AETHERIS')).toBeInTheDocument();
    expect(screen.getByText('Environment-Agnostic Orchestrator')).toBeInTheDocument();
  });

  it('renders service cards', () => {
    render(<App />);
    expect(screen.getByText('Nginx Proxy Manager')).toBeInTheDocument();
    expect(screen.getByText('Portainer')).toBeInTheDocument();
    expect(screen.getByText('Grafana')).toBeInTheDocument();
  });

  it('toggles dark mode', () => {
    render(<App />);

    const toggleButton = screen.getByRole('button', { name: /toggle dark mode/i });

    // Initially should be light mode (mock matchMedia returns false)
    expect(document.documentElement.classList.contains('dark')).toBe(false);

    // Click toggle
    fireEvent.click(toggleButton);
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(localStorage.getItem('theme')).toBe('dark');

    // Click toggle again
    fireEvent.click(toggleButton);
    expect(document.documentElement.classList.contains('dark')).toBe(false);
    expect(localStorage.getItem('theme')).toBe('light');
  });
});
