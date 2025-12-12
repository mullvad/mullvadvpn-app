import { Container } from '../../../../../lib/components';

export type AppUpgradeHeaderProps = {
  children: React.ReactNode;
};

export function AppUpgradeHeader({ children }: AppUpgradeHeaderProps) {
  return (
    <Container indent="medium" flexDirection="column" gap="small">
      {children}
    </Container>
  );
}
