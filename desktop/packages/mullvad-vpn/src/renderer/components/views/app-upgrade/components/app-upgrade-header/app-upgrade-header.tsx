import { Container } from '../../../../../lib/components';

export type AppUpgradeHeaderProps = {
  children: React.ReactNode;
};

export function AppUpgradeHeader({ children }: AppUpgradeHeaderProps) {
  return (
    <Container size="4" $flexDirection="column" $gap="small">
      {children}
    </Container>
  );
}
