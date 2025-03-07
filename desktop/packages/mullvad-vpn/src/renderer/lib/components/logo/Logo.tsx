import { Flex } from '../flex';
import { Image } from '../image';

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
      return <Image source="logo-icon" height={logoSize} />;
    }
    case 'text': {
      const textSize = textSizes[sizeProp];
      return <Image source="logo-text" height={textSize} />;
    }
    case 'both': {
      const logoSize = logoSizes[sizeProp];
      const textSize = textSizes[sizeProp];
      return (
        <Flex $flex={1} $alignItems="center" $gap="small">
          <Image source="logo-icon" height={logoSize} />
          <Image source="logo-text" height={textSize} />
        </Flex>
      );
    }
  }
};
