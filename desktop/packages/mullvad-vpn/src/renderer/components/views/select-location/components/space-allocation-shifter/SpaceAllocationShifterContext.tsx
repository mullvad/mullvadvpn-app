import React from 'react';

type SpaceAllocationShifterProps = Omit<SpaceAllocationShifterProviderProps, 'children'> & {
  sourceHeight: number;
  setSourceHeight: React.Dispatch<React.SetStateAction<number>>;
};

const SpaceAllocationShifter = React.createContext<SpaceAllocationShifterProps | undefined>(
  undefined,
);

export const useSpaceAllocationShifter = (): SpaceAllocationShifterProps => {
  const context = React.useContext(SpaceAllocationShifter);
  if (!context) {
    throw new Error(
      'useSpaceAllocationShifter must be used within a SpaceAllocationShifterProvider',
    );
  }
  return context;
};

type SpaceAllocationShifterProviderProps = React.PropsWithChildren;

export function SpaceAllocationShifterProvider({ children }: SpaceAllocationShifterProviderProps) {
  const [sourceHeight, setSourceHeight] = React.useState(0);

  const value = React.useMemo(
    () => ({
      sourceHeight,
      setSourceHeight,
    }),
    [sourceHeight],
  );

  return (
    <SpaceAllocationShifter.Provider value={value}>{children}</SpaceAllocationShifter.Provider>
  );
}
