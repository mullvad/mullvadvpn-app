import { expect } from 'chai';
import React from 'react';

type ReactElementWithChildren = React.ReactElement<{ children: React.ReactNode }>;

function isReactElementWithChildren(element: unknown): element is ReactElementWithChildren {
  if (React.isValidElement(element)) {
    if (element.props && element.props instanceof Object) {
      return 'children' in element.props;
    }
  }

  return false;
}

export function expectChildrenToMatch(
  element: React.ReactElement<unknown>,
  expectedParts: string[],
) {
  if (!isReactElementWithChildren(element)) {
    throw new Error('React element does not have children on it');
  }

  const elementChildren = React.Children.toArray(element.props.children);
  expect(elementChildren).to.have.lengthOf(expectedParts.length);
  elementChildren.forEach((elementChild, index) => {
    if (!isReactElementWithChildren(elementChild)) {
      throw new Error('React element child does not have children on it');
    }

    expect(elementChild.props.children).to.equal(expectedParts[index]);
  });
}
