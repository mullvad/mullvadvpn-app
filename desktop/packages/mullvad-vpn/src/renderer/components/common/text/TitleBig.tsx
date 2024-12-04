import styled from 'styled-components';

import { typography } from '../../../tokens';
import { Text } from './Text';

export const TitleBig = styled(Text)({
  ...typography['titleBig'],
});
