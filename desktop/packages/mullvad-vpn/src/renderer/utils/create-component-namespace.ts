import React from 'react';

type NamespaceProps = React.JSX.IntrinsicAttributes & { children?: React.ReactNode };

type ComponentNamespace<
  Props extends NamespaceProps,
  NamespaceProperties,
  NamespaceOverrides,
> = React.ComponentType<Props> &
  Omit<NamespaceProperties, keyof NamespaceOverrides> &
  NamespaceOverrides;

export function createComponentNamespace<
  Props extends NamespaceProps,
  NamespaceProperties,
  NamespaceOverrides,
>(
  component: React.ComponentType<Props> & NamespaceProperties,
  overrides?: NamespaceOverrides,
): ComponentNamespace<Props, NamespaceProperties, NamespaceOverrides> {
  const componentNamespace = (props: Props) => React.createElement(component, props);

  componentNamespace.displayName = component.displayName || component.name || 'ComponentNamespace';

  const componentWithNamespace = Object.assign(componentNamespace, component, overrides);

  return componentWithNamespace;
}
