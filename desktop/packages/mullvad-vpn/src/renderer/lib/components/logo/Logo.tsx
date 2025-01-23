import ImageView from '../../../components/ImageView';
import { Spacings } from '../../foundations';
import { Flex } from '../flex';

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
  switch (variant) {
    case 'icon': {
      const logoSize = logoSizes[sizeProp];
      return <ImageView source="logo-icon" height={logoSize} />;
    }
    case 'text': {
      const textSize = textSizes[sizeProp];
      return <ImageView source="logo-text" height={textSize} />;
    }
    case 'both': {
      const logoSize = logoSizes[sizeProp];
      const textSize = textSizes[sizeProp];
      return (
        <Flex $flex={1} $alignItems="center" $gap={Spacings.spacing3}>
          <ImageView source="logo-icon" height={logoSize} />
          <ImageView source="logo-text" height={textSize} />
        </Flex>
      );
    }
  }
};
