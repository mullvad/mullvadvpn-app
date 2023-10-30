import React, { useMemo } from 'react';
import styled from 'styled-components';

import { NonTransientProps } from '../lib/styles';

export interface IImageViewProps
  extends NonTransientProps<IImageMaskProps, 'tintColor' | 'tintHoverColor'> {
  source: string;
  onClick?: (event: React.MouseEvent) => void;
  className?: string;
}

interface IImageMaskProps extends React.HTMLAttributes<HTMLElement> {
  width?: number;
  height?: number;
  disabled?: boolean;
  $tintColor?: string;
  $tintHoverColor?: string;
}

const Wrapper = styled.div({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
});

const ImageMask = styled.div<IImageMaskProps>((props) => {
  const maskWidth = props.width ? `${props.width}px` : 'auto';
  const maskHeight = props.height ? `${props.height}px` : 'auto';
  return {
    maskRepeat: 'no-repeat',
    maskSize: `${maskWidth} ${maskHeight}`,
    maskPosition: 'center',
    lineHeight: 0,
    backgroundColor: props.$tintColor,
    '&&:hover': {
      backgroundColor: (!props.disabled && props.$tintHoverColor) || props.$tintColor,
    },
  };
});

const HiddenImage = styled.img({ visibility: 'hidden' });

export default function ImageView(props: IImageViewProps) {
  const url = props.source.startsWith('data:')
    ? props.source
    : `../../assets/images/${props.source}.svg`;

  // eslint-disable-next-line @typescript-eslint/naming-convention
  const style = useMemo(() => ({ WebkitMaskImage: `url('${url}')` }), [url]);

  if (props.tintColor) {
    const { source: _source, tintColor, tintHoverColor, ...otherProps } = props;
    return (
      <ImageMask
        style={style}
        $tintColor={tintColor}
        $tintHoverColor={tintHoverColor}
        {...otherProps}>
        <HiddenImage src={url} width={props.width} height={props.height} />
      </ImageMask>
    );
  } else {
    const { source: _source, width, height, ...otherProps } = props;
    return (
      <Wrapper {...otherProps}>
        <img src={url} width={width} height={height} aria-hidden={true} />
      </Wrapper>
    );
  }
}
