import styled from 'styled-components';

import { colors } from '../../../tokens';

interface TextProps {
  $color?: keyof typeof colors;
}

export const Text = styled.div<TextProps>((props) => ({
  color: props.$color ? colors[props.$color] : colors.white,
}));
