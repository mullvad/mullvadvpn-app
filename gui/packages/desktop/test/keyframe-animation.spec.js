// @flow

import KeyframeAnimation from '../src/main/keyframe-animation';
import { nativeImage } from 'electron';

describe('lib/keyframe-animation', function() {
  this.timeout(1000);

  const newAnimation = () => {
    const images = [1, 2, 3, 4, 5].map(() => nativeImage.createEmpty());
    const animation = new KeyframeAnimation(images);
    animation.speed = 1;
    return animation;
  };

  it('should play sequence', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
      expect(animation._currentFrame).to.be.equal(4);
      done();
    };

    animation.play();
  });

  it('should play one frame', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([3]);
      expect(animation._currentFrame).to.be.equal(3);
      done();
    };

    animation.play({ startFrame: 3, endFrame: 3 });
  });

  it('should play sequence with custom frames', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([2, 3, 4]);
      expect(animation._currentFrame).to.be.equal(4);
      done();
    };

    animation.play({
      startFrame: 2,
      endFrame: 4,
    });
  });

  it('should play sequence with custom frames in reverse', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2]);
      expect(animation._currentFrame).to.be.equal(2);
      done();
    };

    animation.reverse = true;
    animation.play({
      startFrame: 4,
      endFrame: 2,
    });
  });

  it('should begin from current state starting below range', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
      expect(animation._currentFrame).to.be.equal(4);
      done();
    };

    animation._currentFrame = 0;
    animation._isFirstRun = false;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 3,
      endFrame: 4,
    });
  });

  it('should begin from current state starting below range reverse', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3]);
      expect(animation._currentFrame).to.be.equal(3);
      done();
    };

    animation._currentFrame = 0;
    animation._isFirstRun = false;
    animation.reverse = true;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 3,
      endFrame: 4,
    });
  });

  it('should begin from current state starting above range', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2]);
      expect(animation._currentFrame).to.be.equal(2);
      done();
    };

    animation._currentFrame = 4;
    animation._isFirstRun = false;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 1,
      endFrame: 2,
    });
  });

  it('should begin from current state starting above range reverse', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2, 1]);
      expect(animation._currentFrame).to.be.equal(1);
      done();
    };

    animation._currentFrame = 4;
    animation._isFirstRun = false;
    animation.reverse = true;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 1,
      endFrame: 3,
    });
  });

  it('should play sequence in reverse', (done) => {
    const seq = [];
    const animation = newAnimation();
    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2, 1, 0]);
      expect(animation._currentFrame).to.be.equal(0);
      done();
    };

    animation.reverse = true;
    animation.play();
  });

  it('should play sequence on repeat', (done) => {
    const seq = [];
    const animation = newAnimation();
    const expectedFrames = [0, 1, 2, 3, 4, 0, 1, 2, 3, 4];

    animation.onFrame = () => {
      if (seq.length === expectedFrames.length) {
        animation.stop();
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.play();
  });

  it('should play sequence on repeat in reverse', (done) => {
    const seq = [];
    const animation = newAnimation();
    const expectedFrames = [4, 3, 2, 1, 0, 4, 3, 2, 1, 0];

    animation.onFrame = () => {
      if (seq.length === expectedFrames.length) {
        animation.stop();
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animation.play();
  });

  it('should alternate sequence', (done) => {
    const seq = [];
    const animation = newAnimation();
    const expectedFrames = [0, 1, 2, 3, 4, 3, 2, 1, 0];

    animation.onFrame = () => {
      if (seq.length === expectedFrames.length) {
        animation.stop();
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.alternate = true;
    animation.play();
  });

  it('should alternate reverse sequence', (done) => {
    const seq = [];
    const animation = newAnimation();
    const expectedFrames = [4, 3, 2, 1, 0, 1, 2, 3, 4];

    animation.onFrame = () => {
      if (seq.length === expectedFrames.length) {
        animation.stop();
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animation.alternate = true;
    animation.play();
  });
});
