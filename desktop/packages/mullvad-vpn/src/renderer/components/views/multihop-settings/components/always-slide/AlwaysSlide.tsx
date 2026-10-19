import { messages } from '../../../../../../shared/gettext';
import { Carousel } from '../../../../../lib/components/carousel';
import { FlexColumn } from '../../../../../lib/components/flex-column';

export function AlwaysSlide() {
  return (
    <Carousel.Slides.Slide>
      <Carousel.Slides.Slide.Image
        source="multihop-always-illustration"
        alt={
          // TRANSLATORS: Alt text for an illustration showing traffic when using multihop with the "Always" mode enabled.
          messages.pgettext(
            'accessibility',
            'Illustration showing traffic when using multihop with the "Always" mode enabled',
          )
        }
      />
      <Carousel.Slides.Slide.TextGroup>
        <FlexColumn>
          <Carousel.Slides.Slide.Text variant="labelTinySemiBold">
            {messages.gettext('Always')}
          </Carousel.Slides.Slide.Text>
          <Carousel.Slides.Slide.Text>
            {
              // TRANSLATORS: Information to the user about multihop "Always" mode.
              messages.pgettext(
                'settings-view',
                'Multihop is enabled. Your connection is routed through an entry server before exiting through the selected location.',
              )
            }
          </Carousel.Slides.Slide.Text>
        </FlexColumn>
      </Carousel.Slides.Slide.TextGroup>
    </Carousel.Slides.Slide>
  );
}
