import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from './App';

beforeEach(() => {
  window.localStorage.clear();
});

afterEach(() => {
  window.localStorage.clear();
});

describe('App', () => {
  it('renders Turkish shell labels from the web-only text catalog', () => {
    window.localStorage.setItem('demleme.locale', 'tr');
    render(<App />);

    expect(screen.getByRole('link', { name: 'GRISTORA' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1, name: 'Ana sayfa' })).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: 'Ana gezinme' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Tarifler' })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Tema' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Ana içeriğe geç' })).toBeInTheDocument();
  });

  it('renders English shell labels from the web-only text catalog', () => {
    window.localStorage.setItem('demleme.locale', 'en');
    render(<App />);

    expect(screen.getByRole('link', { name: 'GRISTORA' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1, name: 'Home' })).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: 'Main navigation' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Recipes' })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Theme' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Skip to content' })).toBeInTheDocument();
  });
});
