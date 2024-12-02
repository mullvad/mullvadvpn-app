import styled from 'styled-components';

import { Colors } from '../../../tokens';

interface TextProps {
  $color?: Colors;
}

export const Text = styled.div<TextProps>(({ $color }) => ({
  color: $color ? $color : Colors.white,
}));
