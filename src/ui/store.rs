use cursive::view::{IntoBoxedView, ViewWrapper};
use cursive::views::{BoxedView, NamedView};
use cursive::{wrap_impl, Cursive};
use std::cell::RefCell;

use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use uuid::Uuid;

pub struct Store<T> {
  value: T,
  name: String,
}

pub struct InnerListenerView<'a, T> {
  view: Rc<RefCell<BoxedView>>,
  listener: Box<dyn Fn(&mut BoxedView, &T) + 'a>,
}

pub struct ListenerView<'a, T> {
  view: NamedView<InnerListenerView<'a, T>>,
}

impl<T: 'static> ViewWrapper for ListenerView<'static, T> {
  wrap_impl!(self.view: NamedView<InnerListenerView<'static, T>>);
}

impl<T: 'static> ViewWrapper for InnerListenerView<'static, T> {
  type V = BoxedView;
  fn with_view<F, R>(&self, f: F) -> Option<R>
  where
    F: FnOnce(&Self::V) -> R,
  {
    self
      .view
      .deref()
      .try_borrow()
      .ok()
      .as_ref()
      .map(|v| f(v.deref()))
  }
  fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
  where
    F: FnOnce(&mut Self::V) -> R,
  {
    self
      .view
      .deref()
      .try_borrow_mut()
      .ok()
      .map(|mut v| f(v.deref_mut()))
  }
}

impl<'a, T: 'static> ListenerView<'a, T> {
  pub fn new<V: IntoBoxedView>(
    view: V,
    store: &Store<T>,
    listener: impl Fn(&mut BoxedView, &T) + 'a,
  ) -> Self {
    let view = BoxedView::boxed(view);
    let name = format!("InnerListenerView-{}", store.get_name());
    let view = InnerListenerView::new(view, listener);
    let named = NamedView::new(name, view);
    ListenerView { view: named }
  }
}

impl<'a, T: 'static> InnerListenerView<'a, T> {
  pub fn new(view: BoxedView, listener: impl Fn(&mut BoxedView, &T) + 'a) -> Self {
    InnerListenerView {
      listener: Box::new(listener),
      view: Rc::new(RefCell::new(view)),
    }
  }
}

trait InvokeView<T> {
  fn invoke(&self, value: &T);
}

impl<'a, T> InvokeView<T> for InnerListenerView<'a, T> {
  fn invoke(&self, value: &T) {
    println!("Invoking!");
    match self.view.try_borrow_mut() {
      Ok(mut v) => {
        let child = v.deref_mut();
        (self.listener)(child, value);
      }
      Err(err) => {
        eprintln!("Couldn't borrow: {:?}", err);
      }
    };
  }
}

impl<T: 'static> Store<T> {
  pub fn new(initial_value: T) -> Self {
    Store {
      value: initial_value,
      name: Uuid::new_v4().to_string(),
    }
  }
  pub fn get_name(&self) -> String {
    self.name.clone()
  }
  pub fn use_store(&mut self, _siv: &mut Cursive, view: &mut ListenerView<'static, T>) {
    let value = &self.value;
    let named_view: &mut NamedView<InnerListenerView<'static, T>> = &mut view.view;
    let mut inner_listener = named_view.get_mut();
    let inner_listener: &mut InnerListenerView<'static, T> = inner_listener.deref_mut();
    inner_listener.invoke(value);
  }
  pub fn set(&mut self, siv: &mut Cursive, value: T) {
    println!("Going to talk to all our listeners!");
    self.value = value;
    let value = &self.value;
    siv.call_on_all_named(
      &format!("InnerListenerView-{}", &self.name),
      move |view: &mut InnerListenerView<T>| {
        println!("Invoking a listener");
        view.invoke(value);
      },
    );
  }
  pub fn get(&self) -> &T {
    &self.value
  }
}
