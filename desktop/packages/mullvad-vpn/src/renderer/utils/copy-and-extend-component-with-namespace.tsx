export function copyComponentWithNamespace<P extends object, Subcomponents extends object>(
  Component: React.ComponentType<P>,
  args: Subcomponents,
): React.ComponentType<P> & Subcomponents {
  const CopiedComponent = (props: P) => <Component {...props} />;
  CopiedComponent.displayName = Component.displayName || Component.name;

  return Object.assign(CopiedComponent, args);
}
