import styled, { css } from 'styled-components';

import { colors, Radius } from '../../foundations';
import { Flex } from '../flex';
import { FeatureIndicatorText } from './components';
import { FeatureIndicatorProvider } from './FeatureIndicatorContext';

export type FeatureIndicatorProps = {
  variant?: 'primary' | 'transparent';
} & React.ComponentPropsWithRef<'button'>;

const styles = {
  radius: Radius.radius4,
  variants: {
    primary: {
      backgroundColor: colors.blue10,
      borderColor: colors.blue,
      borderColorHover: colors.whiteAlpha80,
      borderColorPressed: colors.white,
    },
    transparent: {
      backgroundColor: 'transparent',
      borderColor: 'transparent',
      borderColorHover: 'transparent',
      borderColorPressed: 'transparent',
    },
  },
};

const StyledFeatureIndicator = styled.button<{
  $variant: FeatureIndicatorProps['variant'];
  $clickable?: boolean;
}>`
  ${({ $variant: variantProp = 'primary', $clickable }) => {
    const variant = styles.variants[variantProp];
    return css`
      display: flex;
      align-items: center;

      border-radius: ${Radius.radius8};
      background: ${variant.backgroundColor};
      border: 1px solid ${variant.borderColor};

      ${() => {
        if ($clickable) {
          return css`
            &&:not(:disabled):hover {
              border-color: ${variant.borderColorHover};
            }
            &&:not(:disabled):active {
              border-color: ${variant.borderColorPressed};
            }
          `;
        }
        return null;
      }}

      &&:disabled {
        background: var(--disabled);
      }
      &&:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: -2px;
      }
    `;
  }}
`;

const StyledFlex = styled(Flex)`
  padding: 2px 8px;
`;

function FeatureIndicator({
  ref,
  variant,
  children,
  disabled,
  style,
  onClick,
  ...props
}: FeatureIndicatorProps) {
  const clickable = !disabled && !!onClick;
  return (
    <FeatureIndicatorProvider disabled={disabled}>
      <StyledFeatureIndicator
        ref={ref}
        $variant={variant}
        $clickable={clickable}
        disabled={disabled}
        onClick={onClick}
        {...props}>
        <StyledFlex flex={1} alignItems="center">
          {children}
        </StyledFlex>
      </StyledFeatureIndicator>
    </FeatureIndicatorProvider>
  );
}

const FeatureIndicatorNamespace = Object.assign(FeatureIndicator, {
  Text: FeatureIndicatorText,
});

export { FeatureIndicatorNamespace as FeatureIndicator };
