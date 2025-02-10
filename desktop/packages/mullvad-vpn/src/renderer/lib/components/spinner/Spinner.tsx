import styled from 'styled-components';

export interface SpinnerProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  size?: 'small' | 'medium' | 'big';
  alt?: string;
  className?: string;
}

const StyledSpinner = styled.img<{ $size: number }>`
  width: ${({ $size }) => $size}px;
  height: ${({ $size }) => $size}px;
`;

const sizes = {
  small: 16,
  medium: 24,
  big: 48,
};

const PATH_PREFIX = process.env.NODE_ENV === 'development' ? '../' : '';

export const Spinner = ({ size = 'medium', ...props }: SpinnerProps) => {
  const src = `${PATH_PREFIX}assets/images/spinner.svg`;
  return <StyledSpinner src={src} $size={sizes[size]} {...props} />;
};
