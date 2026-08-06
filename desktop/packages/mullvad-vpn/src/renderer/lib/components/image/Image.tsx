import React from 'react';
import styled from 'styled-components';

export type ImageProps = Omit<React.ComponentPropsWithRef<'img'>, 'src'> & {
  source: string;
};

export const StyledImage = styled.img``;

export const Image = ({ source, ...props }: ImageProps) => {
  const url = source.startsWith('data:') ? source : `assets/images/${source}.svg`;

  return <StyledImage src={url} {...props} />;
};
