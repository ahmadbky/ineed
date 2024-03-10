use std::{
    io::{self, BufRead, Write},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::ControlFlow,
    str::FromStr,
};

pub struct MaxTries<P> {
    prompt: P,
    current: usize,
    max: usize,
}

pub struct Then<A, B, O> {
    first: A,
    then: B,
    _marker: PhantomData<O>,
}

pub trait ThenExt {
    type RawOutput;
}

impl<A, B, O> ThenExt for Then<A, B, O>
where
    A: Promptable,
    B: Promptable,
{
    type RawOutput = (<A as Promptable>::Output, <B as Promptable>::Output);
}

pub struct Until<P, F> {
    prompt: P,
    until: F,
}

pub struct Map<P, F> {
    prompt: P,
    map: F,
}

pub trait FromOutput<Output> {
    fn from_output(output: Output) -> Self;
}

impl<T> FromOutput<T> for T {
    fn from_output(output: T) -> Self {
        output
    }
}

macro_rules! impl_from_output {
    ($(
        ($($From:tt)*) into ($($Into:tt)*);
    )*) => {$(
        const _: () = {
            #[automatically_derived]
            impl<$($Into)*> FromOutput<($($From)*)> for ($($Into)*) {
                #[allow(non_snake_case)]
                #[inline(always)]
                fn from_output(($($From)*): ($($From)*)) -> Self {
                    ($($Into)*)
                }
            }
        };
    )*}
}

impl_from_output! {
    ((A, B), C) into (A, B, C);
    (((A, B), C), D) into (A, B, C, D);
    ((((A, B), C), D), E) into (A, B, C, D, E);
    (((((A, B), C), D), E), F) into (A, B, C, D, E, F);
    ((((((A, B), C), D), E), F), G) into (A, B, C, D, E, F, G);
    (((((((A, B), C), D), E), F), G), H) into (A, B, C, D, E, F, G, H);
    ((((((((A, B), C), D), E), F), G), H), I) into (A, B, C, D, E, F, G, H, I);
    (((((((((A, B), C), D), E), F), G), H), I), J) into (A, B, C, D, E, F, G, H, I, J);
}

pub trait Promptable: Sized {
    type Output;

    fn prompt_once<R, W>(&mut self, read: R, write: W) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write;

    fn prompt_with<R, W>(mut self, mut read: R, mut write: W) -> io::Result<Self::Output>
    where
        R: BufRead,
        W: Write,
    {
        loop {
            match self.prompt_once(&mut read, &mut write)? {
                ControlFlow::Break(out) => return Ok(out),
                _ => (),
            }
        }
    }

    fn prompt(self) -> io::Result<Self::Output> {
        self.prompt_with(io::stdin().lock(), io::stdout())
    }

    fn max_tries(self, max: usize) -> MaxTries<Self> {
        MaxTries {
            prompt: self,
            current: 0,
            max,
        }
    }

    fn then<P, O>(self, p: P) -> Then<Self, P, O>
    where
        P: Promptable,
        O: FromOutput<<Then<Self, P, O> as ThenExt>::RawOutput>,
    {
        Then {
            first: self,
            then: p,
            _marker: PhantomData,
        }
    }

    fn until<F>(self, until: F) -> Until<Self, F>
    where
        F: FnMut(&Self::Output) -> bool,
    {
        Until {
            prompt: self,
            until,
        }
    }

    fn map<F, T>(self, map: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> T,
    {
        Map { prompt: self, map }
    }
}

impl<P, F, T> Promptable for Map<P, F>
where
    P: Promptable,
    F: FnMut(<P as Promptable>::Output) -> T,
{
    type Output = T;

    fn prompt_once<R, W>(&mut self, read: R, write: W) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        self.prompt.prompt_once(read, write).map(|flow| match flow {
            ControlFlow::Break(val) => ControlFlow::Break((self.map)(val)),
            ControlFlow::Continue(_) => ControlFlow::Continue(()),
        })
    }
}

impl<P, F> Promptable for Until<P, F>
where
    P: Promptable,
    F: FnMut(&<P as Promptable>::Output) -> bool,
{
    type Output = <P as Promptable>::Output;

    fn prompt_once<R, W>(&mut self, read: R, write: W) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        self.prompt.prompt_once(read, write).map(|flow| match flow {
            ControlFlow::Break(val) if (self.until)(&val) => ControlFlow::Break(val),
            other => other,
        })
    }
}

impl<A, B, O> Promptable for Then<A, B, O>
where
    A: Promptable,
    B: Promptable,
    O: FromOutput<<Self as ThenExt>::RawOutput>,
{
    type Output = O;

    fn prompt_once<R, W>(&mut self, mut read: R, mut write: W) -> io::Result<ControlFlow<O>>
    where
        R: BufRead,
        W: Write,
    {
        let ControlFlow::Break(a) = self.first.prompt_once(&mut read, &mut write)? else {
            return Ok(ControlFlow::Continue(()));
        };

        let b = loop {
            if let ControlFlow::Break(b) = self.then.prompt_once(&mut read, &mut write)? {
                break b;
            }
        };

        Ok(ControlFlow::Break(FromOutput::from_output((a, b))))
    }
}

#[derive(thiserror::Error, Debug)]
#[error("max tries exceeded")]
pub struct MaxTriesExceeded;

impl<P> Promptable for MaxTries<P>
where
    P: Promptable,
{
    type Output = Result<<P as Promptable>::Output, MaxTriesExceeded>;

    fn prompt_once<R, W>(&mut self, read: R, write: W) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        self.current += 1;
        if self.current > self.max {
            return Ok(ControlFlow::Break(Err(MaxTriesExceeded)));
        }

        self.prompt.prompt_once(read, write).map(|flow| match flow {
            ControlFlow::Break(out) => ControlFlow::Break(Ok(out)),
            ControlFlow::Continue(_) => ControlFlow::Continue(()),
        })
    }
}

struct WrittenInner<'a> {
    msg: Option<&'a str>,
    prefix: &'a str,
}

impl<'a> WrittenInner<'a> {
    fn new(msg: &'a str, prefix: &'a str) -> Self {
        Self {
            msg: Some(msg),
            prefix,
        }
    }

    fn prompt<R, W>(&mut self, mut read: R, mut write: W) -> io::Result<String>
    where
        R: BufRead,
        W: Write,
    {
        if let Some(msg) = self.msg.take() {
            write!(write, "{msg}")?;
        }

        write!(write, "{}", self.prefix)?;
        write.flush()?;

        let mut s = String::new();
        read.read_line(&mut s)?;

        Ok(s.trim().to_owned())
    }
}

pub struct Bool<'a> {
    inner: WrittenInner<'a>,
}

pub fn bool<'a>(msg: &'a str, prefix: &'a str) -> Bool<'a> {
    Bool {
        inner: WrittenInner::new(msg, prefix),
    }
}

const TRUE_INPUTS: &[&str] = &["yes", "yep", "true"];
const FALSE_INPUTS: &[&str] = &["noppe", "nah", "false"];

impl Promptable for Bool<'_> {
    type Output = bool;

    fn prompt_once<R, W>(&mut self, read: R, write: W) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let input = self.inner.prompt(read, write)?;
        Ok(match () {
            _ if TRUE_INPUTS.iter().any(|s| s.contains(&input)) => ControlFlow::Break(true),
            _ if FALSE_INPUTS.iter().any(|s| s.contains(&input)) => ControlFlow::Break(false),
            _ => ControlFlow::Continue(()),
        })
    }
}

pub struct Written<'a, T> {
    inner: WrittenInner<'a>,
    _marker: PhantomData<T>,
}

pub fn written<'a, T>(msg: &'a str, prefix: &'a str) -> Written<'a, T> {
    Written {
        inner: WrittenInner::new(msg, prefix),
        _marker: PhantomData,
    }
}

impl<T> Promptable for Written<'_, T>
where
    T: FromStr,
{
    type Output = T;

    fn prompt_once<R, W>(&mut self, read: R, write: W) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let input = self.inner.prompt(read, write)?;
        match input.parse() {
            Ok(out) => Ok(ControlFlow::Break(out)),
            Err(_) => Ok(ControlFlow::Continue(())),
        }
    }
}

pub struct Selected<'a, const N: usize, T> {
    msgs: Option<[&'a str; N]>,
    values: [Option<T>; N],
    prefix: &'a str,
}

impl<const N: usize, T> Promptable for Selected<'_, N, T> {
    type Output = T;

    fn prompt_once<R, W>(
        &mut self,
        mut read: R,
        mut write: W,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        if let Some(list) = self.msgs.take() {
            for (msg, i) in list.into_iter().zip(1..) {
                writeln!(write, "[{i}] - {msg}")?;
            }
        }
        write!(write, "{}", self.prefix)?;
        write.flush()?;

        let mut s = String::new();
        read.read_line(&mut s)?;
        let i = match s.trim().parse::<usize>() {
            Ok(i) if i >= 1 && i <= self.values.len() => i,
            _ => return Ok(ControlFlow::Continue(())),
        };

        match self.values[i - 1].take() {
            Some(out) => Ok(ControlFlow::Break(out)),
            None => Ok(ControlFlow::Continue(())),
        }
    }
}

pub fn selected<'a, const N: usize, T>(
    list: [(&'a str, T); N],
    prefix: &'a str,
) -> Selected<'a, N, T> {
    let mut msgs = unsafe { MaybeUninit::<[MaybeUninit<&'a str>; N]>::uninit().assume_init() };
    let mut values = unsafe { MaybeUninit::<[MaybeUninit<Option<T>>; N]>::uninit().assume_init() };

    for (i, (msg, value)) in list.into_iter().enumerate() {
        msgs[i].write(msg);
        values[i].write(Some(value));
    }

    let msgs = msgs.map(|s| unsafe { s.assume_init() });
    let values = values.map(|v| unsafe { v.assume_init() });

    Selected {
        msgs: Some(msgs),
        values,
        prefix,
    }
}
