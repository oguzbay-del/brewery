import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from './App';

const originalTitle = document.title;

beforeEach(() => {
  document.title = 'Demleme';
});

afterEach(() => {
  document.title = originalTitle;
});

describe('App', () => {
  it('uses the document title as the default page heading', () => {
    render(<App />);
    expect(screen.getByRole('heading', { level: 1, name: document.title })).toBeInTheDocument();
  });
});
