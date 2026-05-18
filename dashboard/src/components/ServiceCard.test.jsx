import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import ServiceCard from './ServiceCard';

describe('ServiceCard Component', () => {
  const mockProps = {
    title: 'Test Service',
    description: 'This is a test description.',
    url: 'http://localhost:9999',
    port: '9999',
  };

  it('renders the card with correct title and description', () => {
    render(<ServiceCard {...mockProps} />);
    expect(screen.getByText('Test Service')).toBeInTheDocument();
    expect(screen.getByText('This is a test description.')).toBeInTheDocument();
  });

  it('renders the correct port', () => {
    render(<ServiceCard {...mockProps} />);
    expect(screen.getByText('Port 9999')).toBeInTheDocument();
  });

  it('has the correct href attribute', () => {
    render(<ServiceCard {...mockProps} />);
    const linkElement = screen.getByRole('link');
    expect(linkElement).toHaveAttribute('href', 'http://localhost:9999');
  });
});
