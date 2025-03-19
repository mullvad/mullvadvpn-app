import styled from 'styled-components';

import { BodySmall } from '../../../../../../../../../lib/components';
import { Flex } from '../../../../../../../../../lib/components';
import { Colors } from '../../../../../../../../../lib/foundations';
import { useChangelog } from '../../hooks';

const StyledList = styled(Flex)`
  list-style-type: disc;
  padding-left: 0;
  li {
    margin-left: 1.5em;
  }
`;

export function ChangelogList() {
  const changelog = useChangelog();

  return (
    <StyledList as="ul" $flexDirection="column" $gap="medium">
      {changelog.map((item, i) => (
        <BodySmall as="li" key={i} color={Colors.white60}>
          {item}
        </BodySmall>
      ))}
    </StyledList>
  );
}
