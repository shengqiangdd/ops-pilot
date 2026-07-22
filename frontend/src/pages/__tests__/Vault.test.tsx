import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { VaultPage } from '../Vault';

const mockCheckStatus = vi.fn();
const mockUnlock = vi.fn();
const mockLock = vi.fn();
const mockSetPassphrase = vi.fn();

let mockIsUnlocked = false;
let mockHasPassphrase = false;
let mockError: string | null = null;

vi.mock('../../stores/useVaultStore', () => ({
  useVaultStore: vi.fn(() => ({
    isUnlocked: mockIsUnlocked,
    hasPassphrase: mockHasPassphrase,
    error: mockError,
    checkStatus: mockCheckStatus,
    unlock: mockUnlock,
    lock: mockLock,
    setPassphrase: mockSetPassphrase,
  })),
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockIsUnlocked = false;
  mockHasPassphrase = false;
  mockError = null;
});

describe('VaultPage', () => {
  it('renders vault description', () => {
    render(<VaultPage />);
    expect(screen.getByText('Vault')).toBeInTheDocument();
    expect(screen.getByText(/encrypts your host credentials/)).toBeInTheDocument();
  });

  it('shows set passphrase form when no passphrase exists', () => {
    render(<VaultPage />);
    expect(screen.getByText('Set Vault Passphrase')).toBeInTheDocument();
    // Check for inputs by placeholder/label
    expect(screen.getAllByLabelText(/Password/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Set Passphrase')).toBeInTheDocument();
  });

  it('shows unlock form when passphrase exists but vault is locked', () => {
    mockHasPassphrase = true;
    render(<VaultPage />);
    expect(screen.getByText('Unlock Vault')).toBeInTheDocument();
    expect(screen.getByText('Unlock')).toBeInTheDocument();
  });

  it('shows unlocked state when vault is unlocked', () => {
    mockHasPassphrase = true;
    mockIsUnlocked = true;
    render(<VaultPage />);
    expect(screen.getByText('Vault Unlocked')).toBeInTheDocument();
    expect(screen.getByText('Lock Vault')).toBeInTheDocument();
  });

  it('calls setPassphrase on form submit', async () => {
    const user = userEvent.setup();
    mockSetPassphrase.mockResolvedValue(undefined);

    render(<VaultPage />);

    const passwordInputs = screen.getAllByLabelText(/Password/i);
    await user.type(passwordInputs[0], 'mypassword');

    const passphraseInputs = screen.getAllByLabelText(/Passphrase/i);
    await user.type(passphraseInputs[0], 'testphrase123');
    await user.type(passphraseInputs[1], 'testphrase123');

    await user.click(screen.getByText('Set Passphrase'));

    await waitFor(() => {
      expect(mockSetPassphrase).toHaveBeenCalledWith('mypassword', 'testphrase123', 'testphrase123');
    });
  });

  it('calls unlock on unlock form submit', async () => {
    mockHasPassphrase = true;
    const user = userEvent.setup();
    mockUnlock.mockResolvedValue(undefined);

    render(<VaultPage />);

    const passwordInputs = screen.getAllByLabelText(/Password/i);
    await user.type(passwordInputs[0], 'mypassword');

    const passphraseInputs = screen.getAllByLabelText(/Passphrase/i);
    await user.type(passphraseInputs[0], 'testphrase');

    await user.click(screen.getByText('Unlock'));

    await waitFor(() => {
      expect(mockUnlock).toHaveBeenCalledWith('mypassword', 'testphrase');
    });
  });

  it('calls lock when lock button is clicked', async () => {
    mockHasPassphrase = true;
    mockIsUnlocked = true;
    const user = userEvent.setup();

    render(<VaultPage />);
    await user.click(screen.getByText('Lock Vault'));

    await waitFor(() => {
      expect(mockLock).toHaveBeenCalled();
    });
  });

  it('displays error from store', () => {
    mockError = 'Something went wrong';
    render(<VaultPage />);
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });
});
