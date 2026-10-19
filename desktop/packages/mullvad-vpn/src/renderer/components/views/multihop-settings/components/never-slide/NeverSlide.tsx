import { messages } from '../../../../../../shared/gettext';
import { Carousel } from '../../../../../lib/components/carousel';
import { FlexColumn } from '../../../../../lib/components/flex-column';

export function NeverSlide() {
  return (
    <Carousel.Slides.Slide>
      <Carousel.Slides.Slide.Image
        source="multihop-never-illustration"
        alt={
          // TRANSLATORS: Alt text for an illustration showing traffic without using multihop.
          messages.pgettext('accessibility', 'Illustration showing traffic without using multihop')
        }
      />
      <Carousel.Slides.Slide.TextGroup>
        <FlexColumn>
          <Carousel.Slides.Slide.Text variant="labelTinySemiBold">
            {messages.gettext('Never')}
          </Carousel.Slides.Slide.Text>
          <Carousel.Slides.Slide.Text>
            {
              // TRANSLATORS: Information to the user about multihop "Never" mode.
              messages.pgettext(
                'settings-view',
                'Multihop is disabled. Your selected location must support all active settings in order to establish a connection.',
              )
            }
          </Carousel.Slides.Slide.Text>
        </FlexColumn>
      </Carousel.Slides.Slide.TextGroup>
    </Carousel.Slides.Slide>
  );
}
