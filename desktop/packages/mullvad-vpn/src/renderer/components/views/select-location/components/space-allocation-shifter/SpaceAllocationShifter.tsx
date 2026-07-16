import { SpaceAllocationShifterSource, SpaceAllocationShifterTarget } from './components';
import { SpaceAllocationShifterProvider } from './SpaceAllocationShifterContext';

type SpaceAllocationShifterProps = React.PropsWithChildren;

function SpaceAllocationShifter({ children }: SpaceAllocationShifterProps) {
  return <SpaceAllocationShifterProvider>{children}</SpaceAllocationShifterProvider>;
}

const SpaceAllocationShifterNamespace = Object.assign(SpaceAllocationShifter, {
  Source: SpaceAllocationShifterSource,
  Target: SpaceAllocationShifterTarget,
});

export { SpaceAllocationShifterNamespace as SpaceAllocationShifter };
