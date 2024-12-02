import styled from 'styled-components';

import { Colors, colors, typography } from '../../../tokens';

interface TextProps {
  $color?: Colors;
}

const Text = styled.div<TextProps>(({ $color }) => ({
  color: $color ? $color : colors.white,
}));

export const TitleBig = styled(Text)({
  ...typography['title-big'],
});
