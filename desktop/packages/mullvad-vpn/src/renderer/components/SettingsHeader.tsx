import * as React from 'react';
import styled from 'styled-components';

import { Flex, LabelTinySemiBold, TitleBig } from '../lib/components';

export const HeaderTitle = styled(TitleBig)({
  wordWrap: 'break-word',
  hyphens: 'auto',
});

export const HeaderSubTitle = styled(LabelTinySemiBold).attrs({
  color: 'whiteAlpha60',
})({});

interface SettingsHeaderProps {
  children?: React.ReactNode;
  className?: string;
}

function SettingsHeader(props: SettingsHeaderProps, forwardRef: React.Ref<HTMLDivElement>) {
  return (
    <Flex
      ref={forwardRef}
      flexDirection="column"
      gap="small"
      margin={{ horizontal: 'medium', bottom: 'medium' }}
      className={props.className}>
      {props.children}
    </Flex>
  );
}

export default React.forwardRef(SettingsHeader);
