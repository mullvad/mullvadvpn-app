import styled from 'styled-components';

import { IChangelog } from '../../../shared/ipc-types';
import { BodySmall } from '../../lib/components';
import { Flex } from '../../lib/components';
import { Colors } from '../../lib/foundations';

const StyledList = styled(Flex)`
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
    <StyledList as="ul" $flexDirection="column" $gap="medium">
      {changelog.map((item, i) => (
        <BodySmall as="li" key={`${item}${i}`} color={Colors.white60}>
          {item}
        </BodySmall>
      ))}
    </StyledList>
  );
}
