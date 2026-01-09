import React from 'react';
import { expect } from 'vitest';

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
  expect(elementChildren).toHaveLength(expectedParts.length);
  elementChildren.forEach((elementChild, index) => {
    if (!isReactElementWithChildren(elementChild)) {
      throw new Error('React element child does not have children on it');
    }

    expect(elementChild.props.children).toEqual(expectedParts[index]);
  });
}

export function expectChildrenToMatchElements(
  element: React.ReactElement<unknown>,
  expectedElements: React.ReactElement[],
) {
  if (!isReactElementWithChildren(element)) {
    throw new Error('React element does not have children on it');
  }

  const elementChildren = React.Children.toArray(element.props.children);
  expect(elementChildren).toHaveLength(expectedElements.length);
  elementChildren.forEach((elementChild, index) => {
    if (!isReactElementWithChildren(elementChild)) {
      throw new Error('React element child does not have children on it');
    }

    const expectedElement = expectedElements[index];
    if (!isReactElementWithChildren(expectedElement)) {
      throw new Error('Expected React element does not have children on it');
    }

    expect(elementChild.type).toEqual(expectedElement.type);
    expect(elementChild.props.children).toEqual(expectedElement.props.children);
  });
}

export function createFragment<T extends React.ReactNode>(value: T) {
  return React.createElement(React.Fragment, null, value);
}
