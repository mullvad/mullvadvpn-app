import React from 'react';

export interface ImageProps {
  source: string;
  onClick?: (event: React.MouseEvent) => void;
  width?: number;
  height?: number;
  className?: string;
}

const PATH_PREFIX = process.env.NODE_ENV === 'development' ? '../' : '';

export const Image = ({ source, ...props }: ImageProps) => {
  const url = source.startsWith('data:') ? source : `${PATH_PREFIX}assets/images/${source}.svg`;

  return <img src={url} {...props} />;
};
