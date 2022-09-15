import { expect } from 'chai';
import { it, describe } from 'mocha';
import KeyframeAnimation from '../../src/main/keyframe-animation';

describe('lib/keyframe-animation', function () {
  this.timeout(1000);

  const newAnimation = () => {
    const animation = new KeyframeAnimation();
    animation.speed = 1;
    return animation;
  };

  it('should play sequence', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
      expect(animation.currentFrame).to.be.equal(4);
      done();
    };

    animation.play({ end: 4 });
  });

  it('should play one frame', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([3]);
      expect(animation.currentFrame).to.be.equal(3);
      done();
    };

    animation.play({ start: 3, end: 3 });
  });

  it('should play sequence with custom frames', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([2, 3, 4]);
      expect(animation.currentFrame).to.be.equal(4);
      done();
    };

    animation.play({ start: 2, end: 4 });
  });

  it('should play sequence with custom frames in reverse', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2]);
      expect(animation.currentFrame).to.be.equal(2);
      done();
    };

    animation.play({ start: 4, end: 2 });
  });

  it('should begin from current state starting below range', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
      expect(animation.currentFrame).to.be.equal(4);
      done();
    };

    animation.currentFrame = 0;
    animation.play({ end: 4 });
  });

  it('should begin from current state starting above range', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2]);
      expect(animation.currentFrame).to.be.equal(2);
      done();
    };

    animation.currentFrame = 4;
    animation.play({ end: 2 });
  });

  it('should begin from current state starting above range reverse', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2, 1]);
      expect(animation.currentFrame).to.be.equal(1);
      done();
    };

    animation.currentFrame = 4;
    animation.play({ end: 1 });
  });

  it('should play sequence in reverse', (done) => {
    const seq: number[] = [];
    const animation = newAnimation();
    animation.onFrame = (frame) => {
      seq.push(frame);
    };
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2, 1, 0]);
      expect(animation.currentFrame).to.be.equal(0);
      done();
    };

    animation.play({ start: 4, end: 0 });
  });
});
