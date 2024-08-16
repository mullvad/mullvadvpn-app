import styled from 'styled-components';

import { colors } from '../../../config.json';
import ImageView from '../ImageView';

const Container = styled.button({
  display: 'flex',
  alignItems: 'center',
  width: '100%',
  background: 'none',
  border: 'none',
});

const Chevron = styled(ImageView)({
  [Container + ':hover &&']: {
    backgroundColor: colors.white80,
  },
});

interface IProps {
  pointsUp: boolean;
  onToggle?: () => void;
  className?: string;
}

export default function ConnectionPanelChevron(props: IProps) {
  return (
    <Container
      data-testid="connection-panel-chevron"
      className={props.className}
      onClick={props.onToggle}>
      <Chevron
        source={props.pointsUp ? 'icon-chevron-up' : 'icon-chevron-down'}
        width={24}
        height={24}
        tintColor={colors.white}
      />
    </Container>
  );
}
