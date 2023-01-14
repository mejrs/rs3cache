use std::borrow::Cow;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{
    plumbing::{Consumer, Folder, UnindexedConsumer},
    IterBridge, ParallelBridge, ParallelIterator,
};

pub struct RenderBar<T>
where
    T: Iterator,
    <T as Iterator>::Item: Send,
{
    base: IterBridge<T>,
    progress: ProgressBar,
    #[allow(dead_code, reason = "used for Drop impl")]
    finalize: Finalizer,
}

struct Finalizer(ProgressBar, Cow<'static, str>);

impl Drop for Finalizer {
    fn drop(&mut self) {
        self.0.finish_and_clear();
        self.0.println(format!("    {} {}", style("Rendered").green().bright(), self.1));
    }
}

pub trait Render: Sized + Iterator
where
    <Self as Iterator>::Item: Send,
{
    fn render(self, name: impl Into<Cow<'static, str>>) -> RenderBar<Self>;
}

impl<S: Send, T: ExactSizeIterator<Item = S> + Send> Render for T {
    fn render(self, name: impl Into<Cow<'static, str>>) -> RenderBar<T> {
        let name = name.into();
        let progress = ProgressBar::new(self.len() as u64).with_style(
            ProgressStyle::with_template(&format!(
                "    {} [{{bar:60}}] {{pos}}/{{len}}: {name}",
                style("Rendering").cyan().bright()
            ))
            .unwrap()
            .progress_chars("=> "),
        );
        RenderBar {
            base: self.par_bridge(),
            progress: progress.clone(),
            finalize: Finalizer(progress, name),
        }
    }
}

impl<S: Send, T: Iterator<Item = S> + Send> ParallelIterator for RenderBar<T> {
    type Item = (S, ProgressBar);

    fn drive_unindexed<C: UnindexedConsumer<Self::Item>>(self, consumer: C) -> C::Result {
        let consumer1 = ProgressConsumer::new(consumer, self.progress.clone());
        self.base.drive_unindexed(consumer1)
    }
}

struct ProgressConsumer<C> {
    base: C,
    progress: ProgressBar,
}

impl<C> ProgressConsumer<C> {
    fn new(base: C, progress: ProgressBar) -> Self {
        ProgressConsumer { base, progress }
    }
}

impl<T, C> Consumer<T> for ProgressConsumer<C>
where
    C: Consumer<(T, ProgressBar)>,
{
    type Folder = ProgressFolder<C::Folder>;
    type Reducer = C::Reducer;
    type Result = C::Result;

    fn split_at(self, index: usize) -> (Self, Self, Self::Reducer) {
        let (left, right, reducer) = self.base.split_at(index);
        (
            ProgressConsumer::new(left, self.progress.clone()),
            ProgressConsumer::new(right, self.progress),
            reducer,
        )
    }

    fn into_folder(self) -> Self::Folder {
        ProgressFolder {
            base: self.base.into_folder(),
            progress: self.progress,
        }
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}

impl<T, C> UnindexedConsumer<T> for ProgressConsumer<C>
where
    C: UnindexedConsumer<(T, ProgressBar)>,
{
    fn split_off_left(&self) -> Self {
        ProgressConsumer::new(self.base.split_off_left(), self.progress.clone())
    }

    fn to_reducer(&self) -> Self::Reducer {
        self.base.to_reducer()
    }
}

struct ProgressFolder<C> {
    base: C,
    progress: ProgressBar,
}

impl<T, C> Folder<T> for ProgressFolder<C>
where
    C: Folder<(T, ProgressBar)>,
{
    type Result = C::Result;

    fn consume(self, item: T) -> Self {
        self.progress.inc(1);
        let item = (item, self.progress.clone());
        ProgressFolder {
            base: self.base.consume(item),
            progress: self.progress,
        }
    }

    fn complete(self) -> C::Result {
        self.base.complete()
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}
