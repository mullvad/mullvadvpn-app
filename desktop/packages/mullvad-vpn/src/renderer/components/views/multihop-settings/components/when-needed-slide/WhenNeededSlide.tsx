import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { Carousel } from '../../../../../lib/components/carousel';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { formatHtml } from '../../../../../lib/html-formatter';

const StyledIcon = styled(Icon)`
  display: inline-block;
  vertical-align: middle;
`;

export function WhenNeededSlide() {
  return (
    <Carousel.Slides.Slide>
      <Carousel.Slides.Slide.Image
        source="multihop-when-needed-illustration"
        alt={
          // TRANSLATORS: Alt text for an illustration showing traffic when using multihop with the "When needed" mode enabled.
          messages.pgettext(
            'accessibility',
            'Illustration showing traffic when using multihop with the "When needed" mode enabled',
          )
        }
      />
      <Carousel.Slides.Slide.TextGroup>
        <FlexColumn>
          <Carousel.Slides.Slide.Text variant="labelTinySemiBold">
            {messages.gettext('When needed')}
          </Carousel.Slides.Slide.Text>
          <Carousel.Slides.Slide.Text>
            {messages.pgettext(
              // TRANSLATORS: Information to the user about multihop "When needed" mode.
              'settings-view',
              'Not all our locations/servers support every feature in the app. If your selected location/server doesn’t support the features you’ve enabled, the app will automatically multihop via a compatible server.',
            )}
          </Carousel.Slides.Slide.Text>
        </FlexColumn>
        <Carousel.Slides.Slide.Text>
          {messages.pgettext(
            // TRANSLATORS: Information to the user about multihop "When needed" mode.
            'settings-view',
            'This ensures your connection does not get blocked due to incompatible settings.',
          )}
        </Carousel.Slides.Slide.Text>
        <Carousel.Slides.Slide.Text>
          {formatHtml(
            // TRANSLATORS: Information to the user about multihop "When needed" mode.
            // Available placeholders:
            // TRANSLATORS: <img></img> - Will be replaced by an icon.
            messages.pgettext('settings-view', 'This will be indicated by the <img></img> symbol'),
            {
              img: () => (
                <StyledIcon
                  icon="magic-multihop"
                  aria-label={messages.gettext('Magic multihop')}
                  color="whiteAlpha60"
                  size="tiny"
                />
              ),
            },
          )}
        </Carousel.Slides.Slide.Text>
      </Carousel.Slides.Slide.TextGroup>
    </Carousel.Slides.Slide>
  );
}
