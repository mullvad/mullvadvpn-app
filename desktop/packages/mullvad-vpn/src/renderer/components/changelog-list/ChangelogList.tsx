import styled from 'styled-components';

import { IChangelog } from '../../../shared/ipc-types';
import { BodySmall } from '../../lib/components';
import { spacings } from '../../lib/foundations';

const StyledList = styled.ul`
  display: flex;
  flex-direction: column;
  gap: ${spacings.medium};
  list-style-type: disc;
  padding-left: 0;
  li {
    margin-left: 1.5em;
  }
`;

export type ChangelogListProps = {
  changelog: IChangelog;
};

export function ChangelogList({ changelog }: ChangelogListProps) {
  return (
    <StyledList>
      {changelog.map((item, i) => (
        <BodySmall as="li" key={`${item}${i}`} color="whiteAlpha60">
          {item}
        </BodySmall>
      ))}
    </StyledList>
  );
}
