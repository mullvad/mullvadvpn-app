import React from 'react';

export interface ImageProps {
  source: string;
  onClick?: (event: React.MouseEvent) => void;
  width?: number;
  height?: number;
  className?: string;
}

export const Image = ({ source, ...props }: ImageProps) => {
  const url = source.startsWith('data:') ? source : `../../assets/images/${source}.svg`;

  return <img src={url} {...props} />;
};
