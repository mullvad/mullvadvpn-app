import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { Carousel } from '../../../../../lib/components/carousel';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { formatHtml } from '../../../../../lib/html-formatter';

export function MultihopSlide() {
  return (
    <Carousel.Slides.Slide>
      <Carousel.Slides.Slide.Image
        source="multihop-illustration"
        alt={
          // TRANSLATORS: Alt text for an illustration showing traffic when using multihop.
          messages.pgettext('accessibility', 'Illustration showing traffic when using multihop')
        }
      />
      <Carousel.Slides.Slide.TextGroup>
        <FlexColumn>
          <Carousel.Slides.Slide.Text>
            {formatHtml(
              sprintf(
                // TRANSLATORS: Information to the user about multihop in general.
                // Available placeholders:
                // TRANSLATORS: %(wireGuard)s - Will be replaced by "WireGuard"
                // TRANSLATORS: <br></br> - Will be replaced by a line break.
                messages.pgettext(
                  'settings-view',
                  'Multihop routes your traffic into one %(wireGuard)s server and out another, making it harder to trace. This increases anonymity online at the cost of increased latency. Multihop has three different modes to choose between: <br></br> When needed, Always, and Never.',
                ),
                {
                  wireGuard: strings.wireguard,
                },
              ),
              {
                br: () => <br />,
              },
            )}
          </Carousel.Slides.Slide.Text>
        </FlexColumn>
      </Carousel.Slides.Slide.TextGroup>
    </Carousel.Slides.Slide>
  );
}
