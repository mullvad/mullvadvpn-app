import { expect } from 'chai';
import React from 'react';

type WithChildren = React.ReactElement<{ children?: React.ReactNode }>;

export const expectChildrenToMatch = (element: React.ReactElement, expectedParts: string[]) => {
  const kids = React.Children.toArray((element as WithChildren).props.children);

  expect(kids).to.have.lengthOf(expectedParts.length);
  kids.forEach((kid, index) => {
    expect((kid as WithChildren).props.children).to.equal(expectedParts[index]);
  });
};
