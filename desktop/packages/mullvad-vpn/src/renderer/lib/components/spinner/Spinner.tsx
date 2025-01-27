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

export const Spinner = ({ size = 'medium', ...props }: SpinnerProps) => {
  const src = '../../assets/images/icon-spinner.svg';
  return <StyledSpinner src={src} $size={sizes[size]} {...props} />;
};
