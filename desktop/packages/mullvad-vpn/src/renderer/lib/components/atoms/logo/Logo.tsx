import ImageView from '../../../../components/ImageView';
import { Spacings } from '../../../foundations';
import { Flex } from '../../layout';

export interface LogoProps {
  variant?: 'icon' | 'text' | 'both';
  size?: '1' | '2';
}

const logoSizes = {
  '1': 38,
  '2': 106,
};

const textSizes = {
  '1': 15.4,
  '2': 18,
};

export const Logo = ({ variant = 'icon', size: sizeProp = '1' }: LogoProps) => {
  const logoSize = logoSizes[sizeProp];
  let logo = <></>;
  if (variant === 'icon' || variant === 'both') {
    logo = <ImageView source="logo-icon" height={logoSize} />;
  }
  const textSize = textSizes[sizeProp];
  let text = <></>;
  if (variant === 'text' || variant === 'both') {
    text = <ImageView source="logo-text" height={textSize} />;
  }

  return (
    <Flex $flex={1} $alignItems="center" $gap={Spacings.spacing3}>
      {logo}
      {text}
    </Flex>
  );
};
