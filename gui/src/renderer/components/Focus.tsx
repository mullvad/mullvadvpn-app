import path from 'path';
import React, { useImperativeHandle, useState } from 'react';
import { useLocation } from 'react-router';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { messages } from '../../shared/gettext';

const PageChangeAnnouncer = styled.div({
  width: 0,
  height: 0,
  overflow: 'hidden',
});

export interface IFocusHandle {
  resetFocus(): void;
}

interface IFocusProps {
  children?: React.ReactElement;
}

function Focus(props: IFocusProps, ref: React.Ref<IFocusHandle>) {
  const location = useLocation();
  const [title, setTitle] = useState<string>();

  useImperativeHandle(
    ref,
    () => ({
      resetFocus: () => {
        const pageName = path.basename(location.pathname);
        const titleElement = document.getElementsByTagName('h1')[0];
        const titleContent = titleElement?.textContent ?? pageName;
        setTitle(titleContent);

        const focusElement = titleElement ?? document.getElementsByTagName('header')[0];
        if (focusElement) {
          focusElement.setAttribute('tabindex', '-1');
          focusElement.focus();
        }
      },
    }),
    [location.pathname],
  );

  return (
    <>
      {title && (
        <PageChangeAnnouncer aria-live="polite">
          {
            // TRANSLATORS: This string is used to notify users of screenreaders that the view has
            // TRANSLATORS: changed, usually as a result of pressing a navigation button.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(title)s - page title
            sprintf(messages.pgettext('accessibility', '%(title)s, View loaded'), { title })
          }
        </PageChangeAnnouncer>
      )}
      {props.children}
    </>
  );
}

export default React.memo(React.forwardRef(Focus));
