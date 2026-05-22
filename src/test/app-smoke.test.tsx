import { render, screen } from '@testing-library/react';
import App from '../App';

it('renders the RustFiles shell', () => {
  render(<App />);
  expect(screen.getByRole('application', { name: 'RustFiles' })).toBeInTheDocument();
});
