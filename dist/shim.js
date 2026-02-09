var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });

// build/index.js
import { WorkerEntrypoint as ht } from "cloudflare:workers";
import Q from "./cbdf5b2428d3f9fbb0865a3d96888128365b3cef-index_bg.wasm";
var E = class {
  static {
    __name(this, "E");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, st.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    s.__wbg_containerstartupoptions_free(t, 0);
  }
  get enableInternet() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let t = s.__wbg_get_containerstartupoptions_enableInternet(this.__wbg_ptr);
    return t === 16777215 ? void 0 : t !== 0;
  }
  get entrypoint() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
      let o = s.__wbindgen_add_to_stack_pointer(-16);
      s.__wbg_get_containerstartupoptions_entrypoint(o, this.__wbg_ptr);
      var t = b().getInt32(o + 0, true), e = b().getInt32(o + 4, true), n = gt(t, e).slice();
      return s.__wbindgen_export4(t, e * 4, 4), n;
    } finally {
      s.__wbindgen_add_to_stack_pointer(16);
    }
  }
  get env() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let t = s.__wbg_get_containerstartupoptions_env(this.__wbg_ptr);
    return y(t);
  }
  set enableInternet(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_containerstartupoptions_enableInternet(this.__wbg_ptr, d(t) ? 16777215 : t ? 1 : 0);
  }
  set entrypoint(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let e = dt(t, s.__wbindgen_export), n = h;
    s.__wbg_set_containerstartupoptions_entrypoint(this.__wbg_ptr, e, n);
  }
  set env(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_containerstartupoptions_env(this.__wbg_ptr, i(t));
  }
};
Symbol.dispose && (E.prototype[Symbol.dispose] = E.prototype.free);
var F = class {
  static {
    __name(this, "F");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ct.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    s.__wbg_intounderlyingbytesource_free(t, 0);
  }
  get autoAllocateChunkSize() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    return s.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr) >>> 0;
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw();
    s.intounderlyingbytesource_cancel(t);
  }
  pull(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let e = s.intounderlyingbytesource_pull(this.__wbg_ptr, i(t));
    return y(e);
  }
  start(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.intounderlyingbytesource_start(this.__wbg_ptr, i(t));
  }
  get type() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let t = s.intounderlyingbytesource_type(this.__wbg_ptr);
    return nt[t];
  }
};
Symbol.dispose && (F.prototype[Symbol.dispose] = F.prototype.free);
var j = class {
  static {
    __name(this, "j");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ut.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    s.__wbg_intounderlyingsink_free(t, 0);
  }
  abort(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let e = this.__destroy_into_raw(), n = s.intounderlyingsink_abort(e, i(t));
    return y(n);
  }
  close() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw(), e = s.intounderlyingsink_close(t);
    return y(e);
  }
  write(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let e = s.intounderlyingsink_write(this.__wbg_ptr, i(t));
    return y(e);
  }
};
Symbol.dispose && (j.prototype[Symbol.dispose] = j.prototype.free);
var S = class {
  static {
    __name(this, "S");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, ft.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    s.__wbg_intounderlyingsource_free(t, 0);
  }
  cancel() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let t = this.__destroy_into_raw();
    s.intounderlyingsource_cancel(t);
  }
  pull(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    let e = s.intounderlyingsource_pull(this.__wbg_ptr, i(t));
    return y(e);
  }
};
Symbol.dispose && (S.prototype[Symbol.dispose] = S.prototype.free);
var v = class r {
  static {
    __name(this, "r");
  }
  static __wrap(t) {
    t = t >>> 0;
    let e = Object.create(r.prototype);
    return e.__wbg_ptr = t, e.__wbg_inst = c, N.register(e, { ptr: t, instance: c }, e), e;
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, N.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    s.__wbg_minifyconfig_free(t, 0);
  }
  get css() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    return s.__wbg_get_minifyconfig_css(this.__wbg_ptr) !== 0;
  }
  get html() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    return s.__wbg_get_minifyconfig_html(this.__wbg_ptr) !== 0;
  }
  get js() {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    return s.__wbg_get_minifyconfig_js(this.__wbg_ptr) !== 0;
  }
  set css(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_minifyconfig_css(this.__wbg_ptr, t);
  }
  set html(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_minifyconfig_html(this.__wbg_ptr, t);
  }
  set js(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_minifyconfig_js(this.__wbg_ptr, t);
  }
};
Symbol.dispose && (v.prototype[Symbol.dispose] = v.prototype.free);
var W = class {
  static {
    __name(this, "W");
  }
  __destroy_into_raw() {
    let t = this.__wbg_ptr;
    return this.__wbg_ptr = 0, at.unregister(this), t;
  }
  free() {
    let t = this.__destroy_into_raw();
    s.__wbg_r2range_free(t, 0);
  }
  get length() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
      let n = s.__wbindgen_add_to_stack_pointer(-16);
      s.__wbg_get_r2range_length(n, this.__wbg_ptr);
      var t = b().getInt32(n + 0, true), e = b().getFloat64(n + 8, true);
      return t === 0 ? void 0 : e;
    } finally {
      s.__wbindgen_add_to_stack_pointer(16);
    }
  }
  get offset() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
      let n = s.__wbindgen_add_to_stack_pointer(-16);
      s.__wbg_get_r2range_offset(n, this.__wbg_ptr);
      var t = b().getInt32(n + 0, true), e = b().getFloat64(n + 8, true);
      return t === 0 ? void 0 : e;
    } finally {
      s.__wbindgen_add_to_stack_pointer(16);
    }
  }
  get suffix() {
    try {
      if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
      let n = s.__wbindgen_add_to_stack_pointer(-16);
      s.__wbg_get_r2range_suffix(n, this.__wbg_ptr);
      var t = b().getInt32(n + 0, true), e = b().getFloat64(n + 8, true);
      return t === 0 ? void 0 : e;
    } finally {
      s.__wbindgen_add_to_stack_pointer(16);
    }
  }
  set length(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_r2range_length(this.__wbg_ptr, !d(t), d(t) ? 0 : t);
  }
  set offset(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_r2range_offset(this.__wbg_ptr, !d(t), d(t) ? 0 : t);
  }
  set suffix(t) {
    if (this.__wbg_inst !== void 0 && this.__wbg_inst !== c) throw new Error("Invalid stale object from previous Wasm instance");
    s.__wbg_set_r2range_suffix(this.__wbg_ptr, !d(t), d(t) ? 0 : t);
  }
};
Symbol.dispose && (W.prototype[Symbol.dispose] = W.prototype.free);
function J() {
  c++, m = null, P = null, typeof numBytesDecoded < "u" && (numBytesDecoded = 0), typeof h < "u" && (h = 0), typeof l < "u" && (l = new Array(128).fill(void 0), l = l.concat([void 0, null, true, false]), typeof x < "u" && (x = l.length)), s = new WebAssembly.Instance(Q, K()).exports, s.__wbindgen_start();
}
__name(J, "J");
function G(r2, t, e) {
  let n = s.fetch(i(r2), i(t), i(e));
  return y(n);
}
__name(G, "G");
function D(r2) {
  s.setPanicHook(i(r2));
}
__name(D, "D");
function K() {
  return { __proto__: null, "./index_bg.js": { __proto__: null, __wbg_Error_8c4e43fe74559d73: /* @__PURE__ */ __name(function(t, e) {
    let n = Error(w(t, e));
    return i(n);
  }, "__wbg_Error_8c4e43fe74559d73"), __wbg_String_8f0eb39a4a4c2f66: /* @__PURE__ */ __name(function(t, e) {
    let n = String(_(e)), o = R(n, s.__wbindgen_export, s.__wbindgen_export2), u = h;
    b().setInt32(t + 4, u, true), b().setInt32(t + 0, o, true);
  }, "__wbg_String_8f0eb39a4a4c2f66"), __wbg___wbindgen_debug_string_0bc8482c6e3508ae: /* @__PURE__ */ __name(function(t, e) {
    let n = q(_(e)), o = R(n, s.__wbindgen_export, s.__wbindgen_export2), u = h;
    b().setInt32(t + 4, u, true), b().setInt32(t + 0, o, true);
  }, "__wbg___wbindgen_debug_string_0bc8482c6e3508ae"), __wbg___wbindgen_is_function_0095a73b8b156f76: /* @__PURE__ */ __name(function(t) {
    return typeof _(t) == "function";
  }, "__wbg___wbindgen_is_function_0095a73b8b156f76"), __wbg___wbindgen_is_object_5ae8e5880f2c1fbd: /* @__PURE__ */ __name(function(t) {
    let e = _(t);
    return typeof e == "object" && e !== null;
  }, "__wbg___wbindgen_is_object_5ae8e5880f2c1fbd"), __wbg___wbindgen_is_string_cd444516edc5b180: /* @__PURE__ */ __name(function(t) {
    return typeof _(t) == "string";
  }, "__wbg___wbindgen_is_string_cd444516edc5b180"), __wbg___wbindgen_is_undefined_9e4d92534c42d778: /* @__PURE__ */ __name(function(t) {
    return _(t) === void 0;
  }, "__wbg___wbindgen_is_undefined_9e4d92534c42d778"), __wbg___wbindgen_string_get_72fb696202c56729: /* @__PURE__ */ __name(function(t, e) {
    let n = _(e), o = typeof n == "string" ? n : void 0;
    var u = d(o) ? 0 : R(o, s.__wbindgen_export, s.__wbindgen_export2), a = h;
    b().setInt32(t + 4, a, true), b().setInt32(t + 0, u, true);
  }, "__wbg___wbindgen_string_get_72fb696202c56729"), __wbg___wbindgen_throw_be289d5034ed271b: /* @__PURE__ */ __name(function(t, e) {
    throw new Error(w(t, e));
  }, "__wbg___wbindgen_throw_be289d5034ed271b"), __wbg__wbg_cb_unref_d9b87ff7982e3b21: /* @__PURE__ */ __name(function(t) {
    _(t)._wbg_cb_unref();
  }, "__wbg__wbg_cb_unref_d9b87ff7982e3b21"), __wbg_abort_2f0584e03e8e3950: /* @__PURE__ */ __name(function(t) {
    _(t).abort();
  }, "__wbg_abort_2f0584e03e8e3950"), __wbg_abort_d549b92d3c665de1: /* @__PURE__ */ __name(function(t, e) {
    _(t).abort(_(e));
  }, "__wbg_abort_d549b92d3c665de1"), __wbg_append_a992ccc37aa62dc4: /* @__PURE__ */ __name(function() {
    return f(function(t, e, n, o, u) {
      _(t).append(w(e, n), w(o, u));
    }, arguments);
  }, "__wbg_append_a992ccc37aa62dc4"), __wbg_arrayBuffer_bb54076166006c39: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).arrayBuffer();
      return i(e);
    }, arguments);
  }, "__wbg_arrayBuffer_bb54076166006c39"), __wbg_body_3a0b4437dadea6bf: /* @__PURE__ */ __name(function(t) {
    let e = _(t).body;
    return d(e) ? 0 : i(e);
  }, "__wbg_body_3a0b4437dadea6bf"), __wbg_buffer_26d0910f3a5bc899: /* @__PURE__ */ __name(function(t) {
    let e = _(t).buffer;
    return i(e);
  }, "__wbg_buffer_26d0910f3a5bc899"), __wbg_byobRequest_80e594e6da4e1af7: /* @__PURE__ */ __name(function(t) {
    let e = _(t).byobRequest;
    return d(e) ? 0 : i(e);
  }, "__wbg_byobRequest_80e594e6da4e1af7"), __wbg_byteLength_3417f266f4bf562a: /* @__PURE__ */ __name(function(t) {
    return _(t).byteLength;
  }, "__wbg_byteLength_3417f266f4bf562a"), __wbg_byteOffset_f88547ca47c86358: /* @__PURE__ */ __name(function(t) {
    return _(t).byteOffset;
  }, "__wbg_byteOffset_f88547ca47c86358"), __wbg_caches_2d1c2248ef602e54: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).caches;
      return i(e);
    }, arguments);
  }, "__wbg_caches_2d1c2248ef602e54"), __wbg_call_389efe28435a9388: /* @__PURE__ */ __name(function() {
    return f(function(t, e) {
      let n = _(t).call(_(e));
      return i(n);
    }, arguments);
  }, "__wbg_call_389efe28435a9388"), __wbg_call_4708e0c13bdc8e95: /* @__PURE__ */ __name(function() {
    return f(function(t, e, n) {
      let o = _(t).call(_(e), _(n));
      return i(o);
    }, arguments);
  }, "__wbg_call_4708e0c13bdc8e95"), __wbg_cancel_2c0a0a251ff6b2b7: /* @__PURE__ */ __name(function(t) {
    let e = _(t).cancel();
    return i(e);
  }, "__wbg_cancel_2c0a0a251ff6b2b7"), __wbg_catch_c1f8c7623b458214: /* @__PURE__ */ __name(function(t, e) {
    let n = _(t).catch(_(e));
    return i(n);
  }, "__wbg_catch_c1f8c7623b458214"), __wbg_cause_0fc168d4eaec87cc: /* @__PURE__ */ __name(function(t) {
    let e = _(t).cause;
    return i(e);
  }, "__wbg_cause_0fc168d4eaec87cc"), __wbg_cf_826be5049e21969d: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).cf;
      return d(e) ? 0 : i(e);
    }, arguments);
  }, "__wbg_cf_826be5049e21969d"), __wbg_cf_b8165e79377eeebd: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).cf;
      return d(e) ? 0 : i(e);
    }, arguments);
  }, "__wbg_cf_b8165e79377eeebd"), __wbg_clearTimeout_42d9ccd50822fd3a: /* @__PURE__ */ __name(function(t) {
    let e = clearTimeout(y(t));
    return i(e);
  }, "__wbg_clearTimeout_42d9ccd50822fd3a"), __wbg_clone_ad5e8c30b1bb1ed5: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).clone();
      return i(e);
    }, arguments);
  }, "__wbg_clone_ad5e8c30b1bb1ed5"), __wbg_clone_d074aef9511a6454: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).clone();
      return i(e);
    }, arguments);
  }, "__wbg_clone_d074aef9511a6454"), __wbg_close_06dfa0a815b9d71f: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      _(t).close();
    }, arguments);
  }, "__wbg_close_06dfa0a815b9d71f"), __wbg_close_a79afee31de55b36: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      _(t).close();
    }, arguments);
  }, "__wbg_close_a79afee31de55b36"), __wbg_constructor_ad6c0ed428f55d34: /* @__PURE__ */ __name(function(t) {
    let e = _(t).constructor;
    return i(e);
  }, "__wbg_constructor_ad6c0ed428f55d34"), __wbg_done_57b39ecd9addfe81: /* @__PURE__ */ __name(function(t) {
    return _(t).done;
  }, "__wbg_done_57b39ecd9addfe81"), __wbg_enqueue_2c63f2044f257c3e: /* @__PURE__ */ __name(function() {
    return f(function(t, e) {
      _(t).enqueue(_(e));
    }, arguments);
  }, "__wbg_enqueue_2c63f2044f257c3e"), __wbg_error_9a7fe3f932034cde: /* @__PURE__ */ __name(function(t) {
    console.error(_(t));
  }, "__wbg_error_9a7fe3f932034cde"), __wbg_error_f852e41c69b0bd84: /* @__PURE__ */ __name(function(t, e) {
    console.error(_(t), _(e));
  }, "__wbg_error_f852e41c69b0bd84"), __wbg_fetch_2c1e75cf1cd9a524: /* @__PURE__ */ __name(function(t, e, n, o) {
    let u = _(t).fetch(w(e, n), _(o));
    return i(u);
  }, "__wbg_fetch_2c1e75cf1cd9a524"), __wbg_fetch_6bbc32f991730587: /* @__PURE__ */ __name(function(t) {
    let e = fetch(_(t));
    return i(e);
  }, "__wbg_fetch_6bbc32f991730587"), __wbg_fetch_afb6a4b6cacf876d: /* @__PURE__ */ __name(function(t, e) {
    let n = _(t).fetch(_(e));
    return i(n);
  }, "__wbg_fetch_afb6a4b6cacf876d"), __wbg_fetch_c97461e1e8f610cd: /* @__PURE__ */ __name(function(t, e, n) {
    let o = _(t).fetch(_(e), _(n));
    return i(o);
  }, "__wbg_fetch_c97461e1e8f610cd"), __wbg_getFullYear_30ddf266b7612036: /* @__PURE__ */ __name(function(t) {
    return _(t).getFullYear();
  }, "__wbg_getFullYear_30ddf266b7612036"), __wbg_getReader_48e00749fe3f6089: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).getReader();
      return i(e);
    }, arguments);
  }, "__wbg_getReader_48e00749fe3f6089"), __wbg_get_b3ed3ad4be2bc8ac: /* @__PURE__ */ __name(function() {
    return f(function(t, e) {
      let n = Reflect.get(_(t), _(e));
      return i(n);
    }, arguments);
  }, "__wbg_get_b3ed3ad4be2bc8ac"), __wbg_get_done_1ad1c16537f444c6: /* @__PURE__ */ __name(function(t) {
    let e = _(t).done;
    return d(e) ? 16777215 : e ? 1 : 0;
  }, "__wbg_get_done_1ad1c16537f444c6"), __wbg_get_value_6b77a1b7b90c9200: /* @__PURE__ */ __name(function(t) {
    let e = _(t).value;
    return i(e);
  }, "__wbg_get_value_6b77a1b7b90c9200"), __wbg_has_1a9792a165aba385: /* @__PURE__ */ __name(function() {
    return f(function(t, e, n) {
      return _(t).has(w(e, n));
    }, arguments);
  }, "__wbg_has_1a9792a165aba385"), __wbg_has_d4e53238966c12b6: /* @__PURE__ */ __name(function() {
    return f(function(t, e) {
      return Reflect.has(_(t), _(e));
    }, arguments);
  }, "__wbg_has_d4e53238966c12b6"), __wbg_headers_59a2938db9f80985: /* @__PURE__ */ __name(function(t) {
    let e = _(t).headers;
    return i(e);
  }, "__wbg_headers_59a2938db9f80985"), __wbg_headers_5a897f7fee9a0571: /* @__PURE__ */ __name(function(t) {
    let e = _(t).headers;
    return i(e);
  }, "__wbg_headers_5a897f7fee9a0571"), __wbg_instanceof_Error_8573fe0b0b480f46: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = _(t) instanceof Error;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_Error_8573fe0b0b480f46"), __wbg_instanceof_ReadableStream_8ab3825017e203e9: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = _(t) instanceof ReadableStream;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_ReadableStream_8ab3825017e203e9"), __wbg_instanceof_Response_ee1d54d79ae41977: /* @__PURE__ */ __name(function(t) {
    let e;
    try {
      e = _(t) instanceof Response;
    } catch {
      e = false;
    }
    return e;
  }, "__wbg_instanceof_Response_ee1d54d79ae41977"), __wbg_iterator_6ff6560ca1568e55: /* @__PURE__ */ __name(function() {
    return i(Symbol.iterator);
  }, "__wbg_iterator_6ff6560ca1568e55"), __wbg_length_32ed9a279acd054c: /* @__PURE__ */ __name(function(t) {
    return _(t).length;
  }, "__wbg_length_32ed9a279acd054c"), __wbg_log_6b5ca2e6124b2808: /* @__PURE__ */ __name(function(t) {
    console.log(_(t));
  }, "__wbg_log_6b5ca2e6124b2808"), __wbg_match_387da9660f8c163a: /* @__PURE__ */ __name(function(t, e, n) {
    let o = _(t).match(_(e), _(n));
    return i(o);
  }, "__wbg_match_387da9660f8c163a"), __wbg_match_c6bae7d1f734857f: /* @__PURE__ */ __name(function(t, e, n, o) {
    let u = _(t).match(w(e, n), _(o));
    return i(u);
  }, "__wbg_match_c6bae7d1f734857f"), __wbg_method_a9e9b2fcba5440fb: /* @__PURE__ */ __name(function(t, e) {
    let n = _(e).method, o = R(n, s.__wbindgen_export, s.__wbindgen_export2), u = h;
    b().setInt32(t + 4, u, true), b().setInt32(t + 0, o, true);
  }, "__wbg_method_a9e9b2fcba5440fb"), __wbg_minifyconfig_new: /* @__PURE__ */ __name(function(t) {
    let e = v.__wrap(t);
    return i(e);
  }, "__wbg_minifyconfig_new"), __wbg_name_07a54d72942d5492: /* @__PURE__ */ __name(function(t) {
    let e = _(t).name;
    return i(e);
  }, "__wbg_name_07a54d72942d5492"), __wbg_new_0_73afc35eb544e539: /* @__PURE__ */ __name(function() {
    return i(/* @__PURE__ */ new Date());
  }, "__wbg_new_0_73afc35eb544e539"), __wbg_new_361308b2356cecd0: /* @__PURE__ */ __name(function() {
    let t = new Object();
    return i(t);
  }, "__wbg_new_361308b2356cecd0"), __wbg_new_64284bd487f9d239: /* @__PURE__ */ __name(function() {
    return f(function() {
      let t = new Headers();
      return i(t);
    }, arguments);
  }, "__wbg_new_64284bd487f9d239"), __wbg_new_72b49615380db768: /* @__PURE__ */ __name(function(t, e) {
    let n = new Error(w(t, e));
    return i(n);
  }, "__wbg_new_72b49615380db768"), __wbg_new_b5d9e2fb389fef91: /* @__PURE__ */ __name(function(t, e) {
    try {
      var n = { a: t, b: e }, o = /* @__PURE__ */ __name((a, g) => {
        let p = n.a;
        n.a = 0;
        try {
          return et(p, n.b, a, g);
        } finally {
          n.a = p;
        }
      }, "o");
      let u = new Promise(o);
      return i(u);
    } finally {
      n.a = n.b = 0;
    }
  }, "__wbg_new_b5d9e2fb389fef91"), __wbg_new_b949e7f56150a5d1: /* @__PURE__ */ __name(function() {
    return f(function() {
      let t = new AbortController();
      return i(t);
    }, arguments);
  }, "__wbg_new_b949e7f56150a5d1"), __wbg_new_dca287b076112a51: /* @__PURE__ */ __name(function() {
    return i(/* @__PURE__ */ new Map());
  }, "__wbg_new_dca287b076112a51"), __wbg_new_dd2b680c8bf6ae29: /* @__PURE__ */ __name(function(t) {
    let e = new Uint8Array(_(t));
    return i(e);
  }, "__wbg_new_dd2b680c8bf6ae29"), __wbg_new_from_slice_a3d2629dc1826784: /* @__PURE__ */ __name(function(t, e) {
    let n = new Uint8Array(L(t, e));
    return i(n);
  }, "__wbg_new_from_slice_a3d2629dc1826784"), __wbg_new_no_args_1c7c842f08d00ebb: /* @__PURE__ */ __name(function(t, e) {
    let n = new Function(w(t, e));
    return i(n);
  }, "__wbg_new_no_args_1c7c842f08d00ebb"), __wbg_new_with_byte_offset_and_length_aa261d9c9da49eb1: /* @__PURE__ */ __name(function(t, e, n) {
    let o = new Uint8Array(_(t), e >>> 0, n >>> 0);
    return i(o);
  }, "__wbg_new_with_byte_offset_and_length_aa261d9c9da49eb1"), __wbg_new_with_headers_4dcfc40d5cedf9d2: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = new Headers(_(t));
      return i(e);
    }, arguments);
  }, "__wbg_new_with_headers_4dcfc40d5cedf9d2"), __wbg_new_with_length_a2c39cbe88fd8ff1: /* @__PURE__ */ __name(function(t) {
    let e = new Uint8Array(t >>> 0);
    return i(e);
  }, "__wbg_new_with_length_a2c39cbe88fd8ff1"), __wbg_new_with_opt_buffer_source_and_init_8c10f2615c78809b: /* @__PURE__ */ __name(function() {
    return f(function(t, e) {
      let n = new Response(_(t), _(e));
      return i(n);
    }, arguments);
  }, "__wbg_new_with_opt_buffer_source_and_init_8c10f2615c78809b"), __wbg_new_with_opt_readable_stream_and_init_8a044befefe6d8bb: /* @__PURE__ */ __name(function() {
    return f(function(t, e) {
      let n = new Response(_(t), _(e));
      return i(n);
    }, arguments);
  }, "__wbg_new_with_opt_readable_stream_and_init_8a044befefe6d8bb"), __wbg_new_with_opt_str_and_init_4fbb71523b271b6e: /* @__PURE__ */ __name(function() {
    return f(function(t, e, n) {
      let o = new Response(t === 0 ? void 0 : w(t, e), _(n));
      return i(o);
    }, arguments);
  }, "__wbg_new_with_opt_str_and_init_4fbb71523b271b6e"), __wbg_new_with_str_and_init_a61cbc6bdef21614: /* @__PURE__ */ __name(function() {
    return f(function(t, e, n) {
      let o = new Request(w(t, e), _(n));
      return i(o);
    }, arguments);
  }, "__wbg_new_with_str_and_init_a61cbc6bdef21614"), __wbg_next_3482f54c49e8af19: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).next();
      return i(e);
    }, arguments);
  }, "__wbg_next_3482f54c49e8af19"), __wbg_next_418f80d8f5303233: /* @__PURE__ */ __name(function(t) {
    let e = _(t).next;
    return i(e);
  }, "__wbg_next_418f80d8f5303233"), __wbg_open_0426947b401fcc70: /* @__PURE__ */ __name(function(t, e, n) {
    let o = _(t).open(w(e, n));
    return i(o);
  }, "__wbg_open_0426947b401fcc70"), __wbg_prototypesetcall_bdcdcc5842e4d77d: /* @__PURE__ */ __name(function(t, e, n) {
    Uint8Array.prototype.set.call(L(t, e), _(n));
  }, "__wbg_prototypesetcall_bdcdcc5842e4d77d"), __wbg_put_2970580aa2ebd536: /* @__PURE__ */ __name(function(t, e, n) {
    let o = _(t).put(_(e), _(n));
    return i(o);
  }, "__wbg_put_2970580aa2ebd536"), __wbg_put_dd78970798f2fe40: /* @__PURE__ */ __name(function(t, e, n, o) {
    let u = _(t).put(w(e, n), _(o));
    return i(u);
  }, "__wbg_put_dd78970798f2fe40"), __wbg_queueMicrotask_0aa0a927f78f5d98: /* @__PURE__ */ __name(function(t) {
    let e = _(t).queueMicrotask;
    return i(e);
  }, "__wbg_queueMicrotask_0aa0a927f78f5d98"), __wbg_queueMicrotask_5bb536982f78a56f: /* @__PURE__ */ __name(function(t) {
    queueMicrotask(_(t));
  }, "__wbg_queueMicrotask_5bb536982f78a56f"), __wbg_read_68fd377df67e19b0: /* @__PURE__ */ __name(function(t) {
    let e = _(t).read();
    return i(e);
  }, "__wbg_read_68fd377df67e19b0"), __wbg_releaseLock_aa5846c2494b3032: /* @__PURE__ */ __name(function(t) {
    _(t).releaseLock();
  }, "__wbg_releaseLock_aa5846c2494b3032"), __wbg_resolve_002c4b7d9d8f6b64: /* @__PURE__ */ __name(function(t) {
    let e = Promise.resolve(_(t));
    return i(e);
  }, "__wbg_resolve_002c4b7d9d8f6b64"), __wbg_respond_bf6ab10399ca8722: /* @__PURE__ */ __name(function() {
    return f(function(t, e) {
      _(t).respond(e >>> 0);
    }, arguments);
  }, "__wbg_respond_bf6ab10399ca8722"), __wbg_setTimeout_4ec014681668a581: /* @__PURE__ */ __name(function(t, e) {
    let n = setTimeout(_(t), e);
    return i(n);
  }, "__wbg_setTimeout_4ec014681668a581"), __wbg_set_1eb0999cf5d27fc8: /* @__PURE__ */ __name(function(t, e, n) {
    let o = _(t).set(_(e), _(n));
    return i(o);
  }, "__wbg_set_1eb0999cf5d27fc8"), __wbg_set_3f1d0b984ed272ed: /* @__PURE__ */ __name(function(t, e, n) {
    _(t)[y(e)] = y(n);
  }, "__wbg_set_3f1d0b984ed272ed"), __wbg_set_6cb8631f80447a67: /* @__PURE__ */ __name(function() {
    return f(function(t, e, n) {
      return Reflect.set(_(t), _(e), _(n));
    }, arguments);
  }, "__wbg_set_6cb8631f80447a67"), __wbg_set_body_9a7e00afe3cfe244: /* @__PURE__ */ __name(function(t, e) {
    _(t).body = _(e);
  }, "__wbg_set_body_9a7e00afe3cfe244"), __wbg_set_cache_315a3ed773a41543: /* @__PURE__ */ __name(function(t, e) {
    _(t).cache = rt[e];
  }, "__wbg_set_cache_315a3ed773a41543"), __wbg_set_cc56eefd2dd91957: /* @__PURE__ */ __name(function(t, e, n) {
    _(t).set(L(e, n));
  }, "__wbg_set_cc56eefd2dd91957"), __wbg_set_credentials_c4a58d2e05ef24fb: /* @__PURE__ */ __name(function(t, e) {
    _(t).credentials = _t[e];
  }, "__wbg_set_credentials_c4a58d2e05ef24fb"), __wbg_set_db769d02949a271d: /* @__PURE__ */ __name(function() {
    return f(function(t, e, n, o, u) {
      _(t).set(w(e, n), w(o, u));
    }, arguments);
  }, "__wbg_set_db769d02949a271d"), __wbg_set_headers_bbdfebba19309590: /* @__PURE__ */ __name(function(t, e) {
    _(t).headers = _(e);
  }, "__wbg_set_headers_bbdfebba19309590"), __wbg_set_headers_cfc5f4b2c1f20549: /* @__PURE__ */ __name(function(t, e) {
    _(t).headers = _(e);
  }, "__wbg_set_headers_cfc5f4b2c1f20549"), __wbg_set_ignore_method_645cd2065463e3e3: /* @__PURE__ */ __name(function(t, e) {
    _(t).ignoreMethod = e !== 0;
  }, "__wbg_set_ignore_method_645cd2065463e3e3"), __wbg_set_method_c3e20375f5ae7fac: /* @__PURE__ */ __name(function(t, e, n) {
    _(t).method = w(e, n);
  }, "__wbg_set_method_c3e20375f5ae7fac"), __wbg_set_mode_b13642c312648202: /* @__PURE__ */ __name(function(t, e) {
    _(t).mode = ot[e];
  }, "__wbg_set_mode_b13642c312648202"), __wbg_set_redirect_a7956fa3f817cbbc: /* @__PURE__ */ __name(function(t, e) {
    _(t).redirect = it[e];
  }, "__wbg_set_redirect_a7956fa3f817cbbc"), __wbg_set_signal_f2d3f8599248896d: /* @__PURE__ */ __name(function(t, e) {
    _(t).signal = _(e);
  }, "__wbg_set_signal_f2d3f8599248896d"), __wbg_set_status_fa41f71c4575bca5: /* @__PURE__ */ __name(function(t, e) {
    _(t).status = e;
  }, "__wbg_set_status_fa41f71c4575bca5"), __wbg_signal_d1285ecab4ebc5ad: /* @__PURE__ */ __name(function(t) {
    let e = _(t).signal;
    return i(e);
  }, "__wbg_signal_d1285ecab4ebc5ad"), __wbg_static_accessor_GLOBAL_12837167ad935116: /* @__PURE__ */ __name(function() {
    let t = typeof global > "u" ? null : global;
    return d(t) ? 0 : i(t);
  }, "__wbg_static_accessor_GLOBAL_12837167ad935116"), __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: /* @__PURE__ */ __name(function() {
    let t = typeof globalThis > "u" ? null : globalThis;
    return d(t) ? 0 : i(t);
  }, "__wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f"), __wbg_static_accessor_SELF_a621d3dfbb60d0ce: /* @__PURE__ */ __name(function() {
    let t = typeof self > "u" ? null : self;
    return d(t) ? 0 : i(t);
  }, "__wbg_static_accessor_SELF_a621d3dfbb60d0ce"), __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: /* @__PURE__ */ __name(function() {
    let t = typeof window > "u" ? null : window;
    return d(t) ? 0 : i(t);
  }, "__wbg_static_accessor_WINDOW_f8727f0cf888e0bd"), __wbg_status_89d7e803db911ee7: /* @__PURE__ */ __name(function(t) {
    return _(t).status;
  }, "__wbg_status_89d7e803db911ee7"), __wbg_stringify_8d1cc6ff383e8bae: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = JSON.stringify(_(t));
      return i(e);
    }, arguments);
  }, "__wbg_stringify_8d1cc6ff383e8bae"), __wbg_then_0d9fe2c7b1857d32: /* @__PURE__ */ __name(function(t, e, n) {
    let o = _(t).then(_(e), _(n));
    return i(o);
  }, "__wbg_then_0d9fe2c7b1857d32"), __wbg_then_b9e7b3b5f1a9e1b5: /* @__PURE__ */ __name(function(t, e) {
    let n = _(t).then(_(e));
    return i(n);
  }, "__wbg_then_b9e7b3b5f1a9e1b5"), __wbg_toString_029ac24421fd7a24: /* @__PURE__ */ __name(function(t) {
    let e = _(t).toString();
    return i(e);
  }, "__wbg_toString_029ac24421fd7a24"), __wbg_url_36c39f6580d05409: /* @__PURE__ */ __name(function(t, e) {
    let n = _(e).url, o = R(n, s.__wbindgen_export, s.__wbindgen_export2), u = h;
    b().setInt32(t + 4, u, true), b().setInt32(t + 0, o, true);
  }, "__wbg_url_36c39f6580d05409"), __wbg_url_c484c26b1fbf5126: /* @__PURE__ */ __name(function(t, e) {
    let n = _(e).url, o = R(n, s.__wbindgen_export, s.__wbindgen_export2), u = h;
    b().setInt32(t + 4, u, true), b().setInt32(t + 0, o, true);
  }, "__wbg_url_c484c26b1fbf5126"), __wbg_value_0546255b415e96c1: /* @__PURE__ */ __name(function(t) {
    let e = _(t).value;
    return i(e);
  }, "__wbg_value_0546255b415e96c1"), __wbg_view_6c32e7184b8606ad: /* @__PURE__ */ __name(function(t) {
    let e = _(t).view;
    return d(e) ? 0 : i(e);
  }, "__wbg_view_6c32e7184b8606ad"), __wbg_webSocket_5d50b1a6fab8a49d: /* @__PURE__ */ __name(function() {
    return f(function(t) {
      let e = _(t).webSocket;
      return d(e) ? 0 : i(e);
    }, arguments);
  }, "__wbg_webSocket_5d50b1a6fab8a49d"), __wbindgen_cast_0000000000000001: /* @__PURE__ */ __name(function(t, e) {
    let n = V(t, e, s.__wasm_bindgen_func_elem_6623, tt);
    return i(n);
  }, "__wbindgen_cast_0000000000000001"), __wbindgen_cast_0000000000000002: /* @__PURE__ */ __name(function(t, e) {
    let n = V(t, e, s.__wasm_bindgen_func_elem_2637, Z);
    return i(n);
  }, "__wbindgen_cast_0000000000000002"), __wbindgen_cast_0000000000000003: /* @__PURE__ */ __name(function(t) {
    return i(t);
  }, "__wbindgen_cast_0000000000000003"), __wbindgen_cast_0000000000000004: /* @__PURE__ */ __name(function(t, e) {
    let n = w(t, e);
    return i(n);
  }, "__wbindgen_cast_0000000000000004"), __wbindgen_cast_0000000000000005: /* @__PURE__ */ __name(function(t) {
    let e = BigInt.asUintN(64, t);
    return i(e);
  }, "__wbindgen_cast_0000000000000005"), __wbindgen_object_clone_ref: /* @__PURE__ */ __name(function(t) {
    let e = _(t);
    return i(e);
  }, "__wbindgen_object_clone_ref"), __wbindgen_object_drop_ref: /* @__PURE__ */ __name(function(t) {
    y(t);
  }, "__wbindgen_object_drop_ref") } };
}
__name(K, "K");
function Z(r2, t) {
  s.__wasm_bindgen_func_elem_2646(r2, t);
}
__name(Z, "Z");
function tt(r2, t, e) {
  s.__wasm_bindgen_func_elem_6639(r2, t, i(e));
}
__name(tt, "tt");
function et(r2, t, e, n) {
  s.__wasm_bindgen_func_elem_7145(r2, t, i(e), i(n));
}
__name(et, "et");
var nt = ["bytes"];
var rt = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];
var _t = ["omit", "same-origin", "include"];
var ot = ["same-origin", "no-cors", "cors", "navigate"];
var it = ["follow", "error", "manual"];
var c = 0;
var st = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: r2, instance: t }) => {
  t === c && s.__wbg_containerstartupoptions_free(r2 >>> 0, 1);
});
var ct = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: r2, instance: t }) => {
  t === c && s.__wbg_intounderlyingbytesource_free(r2 >>> 0, 1);
});
var ut = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: r2, instance: t }) => {
  t === c && s.__wbg_intounderlyingsink_free(r2 >>> 0, 1);
});
var ft = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: r2, instance: t }) => {
  t === c && s.__wbg_intounderlyingsource_free(r2 >>> 0, 1);
});
var N = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: r2, instance: t }) => {
  t === c && s.__wbg_minifyconfig_free(r2 >>> 0, 1);
});
var at = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry(({ ptr: r2, instance: t }) => {
  t === c && s.__wbg_r2range_free(r2 >>> 0, 1);
});
function i(r2) {
  x === l.length && l.push(l.length + 1);
  let t = x;
  return x = l[t], l[t] = r2, t;
}
__name(i, "i");
var $ = typeof FinalizationRegistry > "u" ? { register: /* @__PURE__ */ __name(() => {
}, "register"), unregister: /* @__PURE__ */ __name(() => {
}, "unregister") } : new FinalizationRegistry((r2) => {
  r2.instance === c && r2.dtor(r2.a, r2.b);
});
function q(r2) {
  let t = typeof r2;
  if (t == "number" || t == "boolean" || r2 == null) return `${r2}`;
  if (t == "string") return `"${r2}"`;
  if (t == "symbol") {
    let o = r2.description;
    return o == null ? "Symbol" : `Symbol(${o})`;
  }
  if (t == "function") {
    let o = r2.name;
    return typeof o == "string" && o.length > 0 ? `Function(${o})` : "Function";
  }
  if (Array.isArray(r2)) {
    let o = r2.length, u = "[";
    o > 0 && (u += q(r2[0]));
    for (let a = 1; a < o; a++) u += ", " + q(r2[a]);
    return u += "]", u;
  }
  let e = /\[object ([^\]]+)\]/.exec(toString.call(r2)), n;
  if (e && e.length > 1) n = e[1];
  else return toString.call(r2);
  if (n == "Object") try {
    return "Object(" + JSON.stringify(r2) + ")";
  } catch {
    return "Object";
  }
  return r2 instanceof Error ? `${r2.name}: ${r2.message}
${r2.stack}` : n;
}
__name(q, "q");
function bt(r2) {
  r2 < 132 || (l[r2] = x, x = r2);
}
__name(bt, "bt");
function gt(r2, t) {
  r2 = r2 >>> 0;
  let e = b(), n = [];
  for (let o = r2; o < r2 + 4 * t; o += 4) n.push(y(e.getUint32(o, true)));
  return n;
}
__name(gt, "gt");
function L(r2, t) {
  return r2 = r2 >>> 0, O().subarray(r2 / 1, r2 / 1 + t);
}
__name(L, "L");
var m = null;
function b() {
  return (m === null || m.buffer.detached === true || m.buffer.detached === void 0 && m.buffer !== s.memory.buffer) && (m = new DataView(s.memory.buffer)), m;
}
__name(b, "b");
function w(r2, t) {
  return r2 = r2 >>> 0, wt(r2, t);
}
__name(w, "w");
var P = null;
function O() {
  return (P === null || P.byteLength === 0) && (P = new Uint8Array(s.memory.buffer)), P;
}
__name(O, "O");
function _(r2) {
  return l[r2];
}
__name(_, "_");
function f(r2, t) {
  try {
    return r2.apply(this, t);
  } catch (e) {
    s.__wbindgen_export3(i(e));
  }
}
__name(f, "f");
var l = new Array(128).fill(void 0);
l.push(void 0, null, true, false);
var x = l.length;
function d(r2) {
  return r2 == null;
}
__name(d, "d");
function V(r2, t, e, n) {
  let o = { a: r2, b: t, cnt: 1, dtor: e, instance: c }, u = /* @__PURE__ */ __name((...a) => {
    if (o.instance !== c) throw new Error("Cannot invoke closure from previous WASM instance");
    o.cnt++;
    let g = o.a;
    o.a = 0;
    try {
      return n(g, o.b, ...a);
    } finally {
      o.a = g, u._wbg_cb_unref();
    }
  }, "u");
  return u._wbg_cb_unref = () => {
    --o.cnt === 0 && (o.dtor(o.a, o.b), o.a = 0, $.unregister(o));
  }, $.register(u, o, o), u;
}
__name(V, "V");
function dt(r2, t) {
  let e = t(r2.length * 4, 4) >>> 0, n = b();
  for (let o = 0; o < r2.length; o++) n.setUint32(e + 4 * o, i(r2[o]), true);
  return h = r2.length, e;
}
__name(dt, "dt");
function R(r2, t, e) {
  if (e === void 0) {
    let g = A.encode(r2), p = t(g.length, 1) >>> 0;
    return O().subarray(p, p + g.length).set(g), h = g.length, p;
  }
  let n = r2.length, o = t(n, 1) >>> 0, u = O(), a = 0;
  for (; a < n; a++) {
    let g = r2.charCodeAt(a);
    if (g > 127) break;
    u[o + a] = g;
  }
  if (a !== n) {
    a !== 0 && (r2 = r2.slice(a)), o = e(o, n, n = a + r2.length * 3, 1) >>> 0;
    let g = O().subarray(o + a, o + n), p = A.encodeInto(r2, g);
    a += p.written, o = e(o, n, a, 1) >>> 0;
  }
  return h = a, o;
}
__name(R, "R");
function y(r2) {
  let t = _(r2);
  return bt(r2), t;
}
__name(y, "y");
var Y = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
Y.decode();
function wt(r2, t) {
  return Y.decode(O().subarray(r2, r2 + t));
}
__name(wt, "wt");
var A = new TextEncoder();
"encodeInto" in A || (A.encodeInto = function(r2, t) {
  let e = A.encode(r2);
  return t.set(e), { read: r2.length, written: e.length };
});
var h = 0;
var lt = new WebAssembly.Instance(Q, K());
var s = lt.exports;
Error.stackTraceLimit = 100;
var z = false;
function X() {
  D && D(function(r2) {
    let t = new Error("Rust panic: " + r2);
    console.error("Critical", t), z = true;
  });
}
__name(X, "X");
X();
var M = 0;
function H() {
  z && (console.log("Reinitializing Wasm application"), J(), z = false, X(), M++);
}
__name(H, "H");
addEventListener("error", (r2) => {
  B(r2.error);
});
function B(r2) {
  r2 instanceof WebAssembly.RuntimeError && (console.error("Critical", r2), z = true);
}
__name(B, "B");
var U = class extends ht {
  static {
    __name(this, "U");
  }
};
U.prototype.fetch = function(t) {
  return G.call(this, t, this.env, this.ctx);
};
var yt = { set: /* @__PURE__ */ __name((r2, t, e, n) => Reflect.set(r2.instance, t, e, n), "set"), has: /* @__PURE__ */ __name((r2, t) => Reflect.has(r2.instance, t), "has"), deleteProperty: /* @__PURE__ */ __name((r2, t) => Reflect.deleteProperty(r2.instance, t), "deleteProperty"), apply: /* @__PURE__ */ __name((r2, t, e) => Reflect.apply(r2.instance, t, e), "apply"), construct: /* @__PURE__ */ __name((r2, t, e) => Reflect.construct(r2.instance, t, e), "construct"), getPrototypeOf: /* @__PURE__ */ __name((r2) => Reflect.getPrototypeOf(r2.instance), "getPrototypeOf"), setPrototypeOf: /* @__PURE__ */ __name((r2, t) => Reflect.setPrototypeOf(r2.instance, t), "setPrototypeOf"), isExtensible: /* @__PURE__ */ __name((r2) => Reflect.isExtensible(r2.instance), "isExtensible"), preventExtensions: /* @__PURE__ */ __name((r2) => Reflect.preventExtensions(r2.instance), "preventExtensions"), getOwnPropertyDescriptor: /* @__PURE__ */ __name((r2, t) => Reflect.getOwnPropertyDescriptor(r2.instance, t), "getOwnPropertyDescriptor"), defineProperty: /* @__PURE__ */ __name((r2, t, e) => Reflect.defineProperty(r2.instance, t, e), "defineProperty"), ownKeys: /* @__PURE__ */ __name((r2) => Reflect.ownKeys(r2.instance), "ownKeys") };
var I = { construct(r2, t, e) {
  try {
    H();
    let n = { instance: Reflect.construct(r2, t, e), instanceId: M, ctor: r2, args: t, newTarget: e };
    return new Proxy(n, { ...yt, get(o, u, a) {
      o.instanceId !== M && (o.instance = Reflect.construct(o.ctor, o.args, o.newTarget), o.instanceId = M);
      let g = Reflect.get(o.instance, u, a);
      return typeof g != "function" ? g : g.constructor === Function ? new Proxy(g, { apply(p, T, C) {
        H();
        try {
          return p.apply(T, C);
        } catch (k) {
          throw B(k), k;
        }
      } }) : new Proxy(g, { async apply(p, T, C) {
        H();
        try {
          return await p.apply(T, C);
        } catch (k) {
          throw B(k), k;
        }
      } });
    } });
  } catch (n) {
    throw z = true, n;
  }
} };
var vt = new Proxy(U, I);
var It = new Proxy(E, I);
var Rt = new Proxy(F, I);
var Et = new Proxy(j, I);
var Ft = new Proxy(S, I);
var jt = new Proxy(v, I);
var St = new Proxy(W, I);
export {
  It as ContainerStartupOptions,
  Rt as IntoUnderlyingByteSource,
  Et as IntoUnderlyingSink,
  Ft as IntoUnderlyingSource,
  jt as MinifyConfig,
  St as R2Range,
  vt as default
};
//# sourceMappingURL=shim.js.map
