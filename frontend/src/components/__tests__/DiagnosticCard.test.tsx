import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { DiagnosticCard } from '../DiagnosticCard';

describe('DiagnosticCard', () => {
  it('renders with ok status', () => {
    render(<DiagnosticCard name="CPU" status="ok" score={95} icon="🖥️" />);
    expect(screen.getByText('CPU')).toBeInTheDocument();
    expect(screen.getByText('ok')).toBeInTheDocument();
    expect(screen.getByText('95')).toBeInTheDocument();
  });

  it('renders with warning status', () => {
    render(<DiagnosticCard name="Memory" status="warning" score={65} icon="💾" />);
    expect(screen.getByText('Memory')).toBeInTheDocument();
    expect(screen.getByText('warning')).toBeInTheDocument();
  });

  it('renders with critical status', () => {
    render(<DiagnosticCard name="Disk" status="critical" score={30} icon="💿" />);
    expect(screen.getByText('Disk')).toBeInTheDocument();
    expect(screen.getByText('critical')).toBeInTheDocument();
  });

  it('uses category icon for known categories', () => {
    render(<DiagnosticCard name="CPU" status="ok" score={95} icon="🖥️" />);
    expect(screen.getByText('🖥️')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    render(<DiagnosticCard name="CPU" status="ok" score={95} icon="🖥️" onClick={onClick} />);
    await user.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('shows expanded state with ring', () => {
    render(<DiagnosticCard name="CPU" status="ok" score={95} icon="🖥️" expanded />);
    const button = screen.getByRole('button');
    expect(button.className).toContain('ring-2');
  });

  it('renders health label based on score', () => {
    render(<DiagnosticCard name="CPU" status="ok" score={95} icon="🖥️" />);
    expect(screen.getByText('Healthy')).toBeInTheDocument();
  });
});
