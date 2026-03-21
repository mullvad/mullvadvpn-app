import styled, { css } from 'styled-components';

import { Accordion } from '../../../../../../../../../lib/components/accordion';
import type { AccordionContentProps } from '../../../../../../../../../lib/components/accordion/components';
import { spacings } from '../../../../../../../../../lib/foundations';
import { useLocationContext } from '../../../../LocationContext';
import { useEffectScrollOnExpand } from './hooks';

export type LocationAccordionContentProps = AccordionContentProps;

export const StyledLocationAccordionContent = styled(Accordion.Content)<{
  $root?: boolean;
}>`
  ${({ $root }) => {
    return css`
      ${() => {
        if ($root) {
          return css`
            // If accordion content is in the root list item, add some extra bottom margin to separate it from the next location.
            // Target the last child of the accordion to prevent stutters in expand/collapse animation.
            &:last-child > :last-child {
              margin-bottom: ${spacings.tiny};
            }
          `;
        }

        return null;
      }}
    `;
  }}
`;

export const LocationAccordionContent = ({ ...props }: LocationAccordionContentProps) => {
  useEffectScrollOnExpand();
  const { root } = useLocationContext();

  return <StyledLocationAccordionContent $root={root} {...props} />;
};
