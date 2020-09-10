import React, { useCallback } from 'react';
import { useHistory } from 'react-router';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import ImageView from './ImageView';

export enum HeaderBarStyle {
  default = 'default',
  defaultDark = 'defaultDark',
  error = 'error',
  success = 'success',
}

const headerBarStyleColorMap = {
  [HeaderBarStyle.default]: colors.blue,
  [HeaderBarStyle.defaultDark]: colors.darkBlue,
  [HeaderBarStyle.error]: colors.red,
  [HeaderBarStyle.success]: colors.green,
};

const HeaderBarContainer = styled.header({}, (props: { barStyle?: HeaderBarStyle }) => ({
  padding: '12px 16px',
  paddingTop: process.platform === 'darwin' ? '24px' : '12px',
  backgroundColor: headerBarStyleColorMap[props.barStyle ?? HeaderBarStyle.default],
}));

const HeaderBarContent = styled.div({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'flex-end',
  // In views without the brand components we still want the Header to have the same height.
  minHeight: '51px',
});

interface IHeaderBarProps {
  barStyle?: HeaderBarStyle;
  className?: string;
  children?: React.ReactNode;
}

export default function HeaderBar(props: IHeaderBarProps) {
  return (
    <HeaderBarContainer barStyle={props.barStyle} className={props.className}>
      <HeaderBarContent>{props.children}</HeaderBarContent>
    </HeaderBarContainer>
  );
}

const BrandContainer = styled.div({
  display: 'flex',
  flex: 1,
  alignItems: 'center',
});

const Title = styled.h1({
  fontFamily: 'DINPro',
  fontSize: '24px',
  fontWeight: 900,
  lineHeight: '30px',
  color: colors.white80,
  marginLeft: '9px',
});

const Logo = styled(ImageView)({
  margin: '4px 0 3px',
});

export function Brand() {
  return (
    <BrandContainer>
      <Logo width={44} height={44} source="logo-icon" />
      <Title>{messages.pgettext('generic', 'MULLVAD VPN')}</Title>
    </BrandContainer>
  );
}

const HeaderBarSettingsButtonContainer = styled.button({
  cursor: 'default',
  padding: 0,
  marginLeft: 8,
  backgroundColor: 'transparent',
  border: 'none',
});

export function HeaderBarSettingsButton() {
  const history = useHistory();

  const openSettings = useCallback(() => {
    history.push('/settings');
  }, [history]);

  return (
    <HeaderBarSettingsButtonContainer onClick={openSettings}>
      <ImageView
        height={24}
        width={24}
        source="icon-settings"
        tintColor={colors.white80}
        tintHoverColor={colors.white}
      />
    </HeaderBarSettingsButtonContainer>
  );
}
