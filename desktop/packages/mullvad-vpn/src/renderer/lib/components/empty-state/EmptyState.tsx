import { Flex, FlexProps } from '../flex';
import {
  EmptyStateButton,
  EmptyStateStatusIcon,
  EmptyStateSubtitle,
  EmptyStateTextContainer,
  EmptyStateTitle,
} from './components';
import { EmptyStateProvider } from './EmptyStateContext';

type EmptyStateVariant = 'success' | 'error' | 'loading';

export type EmptyStateProps = FlexProps & {
  variant?: EmptyStateVariant;
};

function EmptyState({ variant = 'error', children, ...props }: EmptyStateProps) {
  return (
    <EmptyStateProvider variant={variant}>
      <Flex flexDirection="column" gap="medium" alignItems="center" {...props}>
        {children}
      </Flex>
    </EmptyStateProvider>
  );
}

const EmptyStateNamespace = Object.assign(EmptyState, {
  StatusIcon: EmptyStateStatusIcon,
  Subtitle: EmptyStateSubtitle,
  Title: EmptyStateTitle,
  Button: EmptyStateButton,
  TextContainer: EmptyStateTextContainer,
});

export { EmptyStateNamespace as EmptyState };
