import path from 'path';
import * as React from 'react';
import styled from 'styled-components';

export interface IImageViewProps extends IImageMaskProps {
  onClick?: (event: React.MouseEvent) => void;
  className?: string;
}

interface IImageMaskProps {
  source: string;
  width?: number;
  height?: number;
  disabled?: boolean;
  tintColor?: string;
  tintHoverColor?: string;
}

const Wrapper = styled.div({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'center',
});

const ImageMask = styled.div((props: IImageMaskProps) => {
  const maskWidth = props.width ? `${props.width}px` : 'auto';
  const maskHeight = props.height ? `${props.height}px` : 'auto';
  return {
    maskImage: `url('${props.source}')`,
    maskRepeat: 'no-repeat',
    maskSize: `${maskWidth} ${maskHeight}`,
    maskPosition: 'center',
    lineHeight: 0,
    backgroundColor: props.tintColor,
    ':hover': {
      backgroundColor: (!props.disabled && props.tintHoverColor) || props.tintColor,
    },
  };
});

const HiddenImage = styled.img({ visibility: 'hidden' });

export default function ImageView(props: IImageViewProps) {
  const url = path.isAbsolute(props.source)
    ? props.source
    : `../../assets/images/${props.source}.svg`;

  if (props.tintColor) {
    const { source: _source, ...otherProps } = props;
    return (
      <ImageMask source={url} {...otherProps}>
        <HiddenImage src={url} width={props.width} height={props.height} />
      </ImageMask>
    );
  } else {
    return (
      <Wrapper onClick={props.onClick} className={props.className}>
        <img src={url} width={props.width} height={props.height} />
      </Wrapper>
    );
  }
}
