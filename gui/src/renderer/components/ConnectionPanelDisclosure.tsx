import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { normalText } from './common-styles';
import ImageView from './ImageView';

const Container = styled.div({
  display: 'flex',
  alignItems: 'center',
  width: '100%',
});

const Caption = styled.span(normalText, (props: { open: boolean }) => ({
  fontWeight: 600,
  lineHeight: '20px',
  minWidth: '0px',
  color: props.open ? colors.white : colors.white40,
  [Container + ':hover &']: {
    color: colors.white,
  },
}));

const Chevron = styled(ImageView)({
  [Container + ':hover &']: {
    backgroundColor: colors.white,
  },
});

interface IProps {
  pointsUp: boolean;
  onToggle?: () => void;
  children: React.ReactNode;
  className?: string;
}

export default function ConnectionPanelDisclosure(props: IProps) {
  return (
    <Container className={props.className} onClick={props.onToggle}>
      <Caption open={props.pointsUp}>{props.children}</Caption>
      <Chevron
        source={props.pointsUp ? 'icon-chevron-up' : 'icon-chevron-down'}
        width={24}
        height={24}
        tintColor={colors.white40}
      />
    </Container>
  );
}
