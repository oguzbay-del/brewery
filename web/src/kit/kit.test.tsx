import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { AlarmMark, Callout, StateView, StatusBadge, SummaryCard } from './index';

describe('UI Kit', () => {
  it('exposes Callout text and a distinct warning icon', () => {
    const { container } = render(
      <Callout tone="warning" title="Gecikmiş">
        Ölçüm bekleniyor
      </Callout>,
    );
    expect(screen.getByRole('status')).toHaveTextContent('Gecikmiş');
    expect(screen.getByRole('status')).toHaveTextContent('Ölçüm bekleniyor');
    expect(container.querySelector('[data-tone="warning"] svg')).toHaveAttribute(
      'aria-hidden',
      'true',
    );
  });

  it('names the loading state and exposes its busy status', () => {
    render(<StateView kind="loading" title="Yükleniyor" />);
    const region = screen.getByRole('region', { name: 'Yükleniyor' });
    expect(region).toHaveAttribute('aria-busy', 'true');
    expect(region).toHaveAttribute('data-kind', 'loading');
    expect(within(region).getByRole('status')).toHaveTextContent('Yükleniyor');
  });

  it('names a metric and retains its value and unit', () => {
    render(<SummaryCard label="Sıcaklık" value="19,2" unit="°C" />);
    const region = screen.getByRole('region', { name: 'Sıcaklık' });
    expect(within(region).getByText('19,2')).toBeInTheDocument();
    expect(within(region).getByText('°C')).toBeInTheDocument();
  });

  it('shows a status word and icon without relying on color', () => {
    const { container } = render(<StatusBadge tone="success" label="Hazır" />);
    expect(screen.getByText('Hazır')).toBeVisible();
    expect(container.querySelector('[data-tone="success"] svg')).toBeInTheDocument();
  });

  it('shows alarm priority, shape and word', () => {
    const { container } = render(<AlarmMark level={2} label="Uyarı" />);
    expect(screen.getByText('Uyarı')).toBeVisible();
    expect(container.querySelector('[data-level="2"]')).toHaveTextContent('2');
    expect(container.querySelector('.kit-alarm-shape')).toBeInTheDocument();
  });
});
