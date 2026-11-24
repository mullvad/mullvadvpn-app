import styled, { css } from 'styled-components';

import { colors, Radius } from '../../foundations';
import {
  FeatureIndicatorIcon,
  FeatureIndicatorText,
  StyledFeatureIndicatorText,
} from './components';
import { FeatureIndicatorProvider } from './FeatureIndicatorContext';

export type FeatureIndicatorProps = {
  inactive?: boolean;
} & React.ComponentPropsWithRef<'button'>;

const variables = {
  backgroundColor: colors.blue10,
  backgroundColorInactiveHover: colors.blue40,
  backgroundColorInactivePressed: colors.blue60,
  borderColor: colors.blue,
  borderColorHover: colors.whiteAlpha80,
  borderColorPressed: colors.white,
};

const StyledFeatureIndicator = styled.button<{
  $clickable?: boolean;
  $inactive?: boolean;
}>`
  ${({ $clickable, $inactive }) => {
    return css`
      display: flex;
      width: fit-content;
      align-items: center;
      gap: 2px;

      padding: 2px 8px;

      border-radius: ${Radius.radius8};

      background: ${variables.backgroundColor};

      ${() => {
        if (!$inactive) {
          return css`
            &&:not(:disabled) {
              border: 1px solid ${variables.borderColor};
            }
          `;
        }

        return null;
      }}

      ${() => {
        if ($clickable) {
          if ($inactive) {
            return css`
              &&:hover {
                background: ${variables.backgroundColorInactiveHover};

                > ${StyledFeatureIndicatorText} {
                  color: ${colors.whiteAlpha40};
                }
              }

              &&:active {
                background: ${variables.backgroundColorInactivePressed};

                > ${StyledFeatureIndicatorText} {
                  color: ${colors.whiteAlpha60};
                }
              }
            `;
          }

          return css`
            &&:not(:disabled):hover {
              border-color: ${variables.borderColorHover};
            }
            &&:not(:disabled):active {
              border-color: ${variables.borderColorPressed};
            }
          `;
        }

        return null;
      }}

      &&:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: -2px;
      }
    `;
  }}
`;

function FeatureIndicator({
  ref,
  children,
  disabled,
  inactive,
  style,
  onClick,
  ...props
}: FeatureIndicatorProps) {
  const clickable = !disabled && !!onClick;
  return (
    <FeatureIndicatorProvider disabled={disabled} inactive={inactive}>
      <StyledFeatureIndicator
        ref={ref}
        $clickable={clickable}
        $inactive={inactive}
        disabled={disabled}
        onClick={onClick}
        {...props}>
        {children}
      </StyledFeatureIndicator>
    </FeatureIndicatorProvider>
  );
}

const FeatureIndicatorNamespace = Object.assign(FeatureIndicator, {
  Icon: FeatureIndicatorIcon,
  Text: FeatureIndicatorText,
});

export { FeatureIndicatorNamespace as FeatureIndicator };
