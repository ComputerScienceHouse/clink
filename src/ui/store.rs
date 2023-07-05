use cursive::view::{IntoBoxedView, ViewWrapper};
use cursive::views::{BoxedView, NamedView};
use cursive::View;
use cursive::{wrap_impl, Cursive};
use std::cell::RefCell;

use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use uuid::Uuid;

pub struct Store<T> {
  value: T,
  name: String,
}

type ListenerFn<'a, T> = Box<dyn Fn(&mut BoxedView, &T, &T) + 'a>;
pub struct InnerListenerView<'a, T> {
  view: Rc<RefCell<BoxedView>>,
  listener: ListenerFn<'a, T>,
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
    listener: impl Fn(&mut BoxedView, &T, &T) + 'a,
  ) -> Self {
    let view = BoxedView::boxed(view);
    let name = format!("InnerListenerView-{}", store.get_name());
    let view = InnerListenerView::new(view, listener);
    let named = NamedView::new(name, view);
    ListenerView { view: named }
  }
}

impl<T: 'static> ListenerView<'static, T> {
  pub fn with_child<V: View, F, R>(&mut self, cb: F) -> R
  where
    F: FnOnce(&mut V) -> R,
  {
    self.view.with_view_mut(|v| v.with_child(cb)).unwrap()
  }
}

impl<'a, T: 'static> InnerListenerView<'a, T> {
  pub fn new(view: BoxedView, listener: impl Fn(&mut BoxedView, &T, &T) + 'a) -> Self {
    InnerListenerView {
      listener: Box::new(listener),
      view: Rc::new(RefCell::new(view)),
    }
  }
}

impl<T: 'static> InnerListenerView<'static, T> {
  pub fn with_child<V: View, F, R>(&self, cb: F) -> R
  where
    F: FnOnce(&mut V) -> R,
  {
    let mut v = self.view.try_borrow_mut().unwrap();
    let child: &mut BoxedView = v.deref_mut();
    let child = child.deref_mut();
    if let Some(v) = child.downcast_mut::<V>() {
      cb(v)
    } else {
      panic!(
        "Expected {:?}, found {} instead!",
        std::any::type_name::<V>(),
        child.type_name()
      )
    }
  }
}

trait InvokeView<T> {
  fn invoke(&self, old_value: &T, value: &T);
}

impl<'a, T> InvokeView<T> for InnerListenerView<'a, T> {
  fn invoke(&self, old_value: &T, value: &T) {
    match self.view.try_borrow_mut() {
      Ok(mut v) => {
        let child = v.deref_mut();
        (self.listener)(child, old_value, value);
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
    inner_listener.invoke(value, value);
  }
  pub fn set(&mut self, siv: &mut Cursive, value: T) {
    let old_value = mem::replace(&mut self.value, value);
    let old_value = &old_value;
    let value = &self.value;
    siv.call_on_all_named(
      &format!("InnerListenerView-{}", &self.name),
      move |view: &mut InnerListenerView<T>| {
        view.invoke(old_value, value);
      },
    );
  }
  pub fn get(&self) -> &T {
    &self.value
  }
}
