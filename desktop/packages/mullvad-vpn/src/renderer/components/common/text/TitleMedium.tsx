import styled from 'styled-components';

import { typography } from '../../../tokens';
import { Text } from './Text';

export const TitleMedium = styled(Text)({
  ...typography['title-medium'],
});
