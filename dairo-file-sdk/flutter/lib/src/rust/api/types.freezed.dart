// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$Event {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)
        stateChanged,
    required TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)
        progress,
    required TResult Function(List<String> ids) removed,
    required TResult Function(String id, UploadState state, String? error)
        uploadStateChanged,
    required TResult Function(
            String id, bool hashing, int size, int done, int speed)
        uploadProgress,
    required TResult Function(List<String> ids) uploadRemoved,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult? Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult? Function(List<String> ids)? removed,
    TResult? Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult? Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult? Function(List<String> ids)? uploadRemoved,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult Function(List<String> ids)? removed,
    TResult Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult Function(List<String> ids)? uploadRemoved,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Event_StateChanged value) stateChanged,
    required TResult Function(Event_Progress value) progress,
    required TResult Function(Event_Removed value) removed,
    required TResult Function(Event_UploadStateChanged value)
        uploadStateChanged,
    required TResult Function(Event_UploadProgress value) uploadProgress,
    required TResult Function(Event_UploadRemoved value) uploadRemoved,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Event_StateChanged value)? stateChanged,
    TResult? Function(Event_Progress value)? progress,
    TResult? Function(Event_Removed value)? removed,
    TResult? Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult? Function(Event_UploadProgress value)? uploadProgress,
    TResult? Function(Event_UploadRemoved value)? uploadRemoved,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Event_StateChanged value)? stateChanged,
    TResult Function(Event_Progress value)? progress,
    TResult Function(Event_Removed value)? removed,
    TResult Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult Function(Event_UploadProgress value)? uploadProgress,
    TResult Function(Event_UploadRemoved value)? uploadRemoved,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $EventCopyWith<$Res> {
  factory $EventCopyWith(Event value, $Res Function(Event) then) =
      _$EventCopyWithImpl<$Res, Event>;
}

/// @nodoc
class _$EventCopyWithImpl<$Res, $Val extends Event>
    implements $EventCopyWith<$Res> {
  _$EventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$Event_StateChangedImplCopyWith<$Res> {
  factory _$$Event_StateChangedImplCopyWith(_$Event_StateChangedImpl value,
          $Res Function(_$Event_StateChangedImpl) then) =
      __$$Event_StateChangedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String id, CacheMode mode, DownloadState state, String? error});
}

/// @nodoc
class __$$Event_StateChangedImplCopyWithImpl<$Res>
    extends _$EventCopyWithImpl<$Res, _$Event_StateChangedImpl>
    implements _$$Event_StateChangedImplCopyWith<$Res> {
  __$$Event_StateChangedImplCopyWithImpl(_$Event_StateChangedImpl _value,
      $Res Function(_$Event_StateChangedImpl) _then)
      : super(_value, _then);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? mode = null,
    Object? state = null,
    Object? error = freezed,
  }) {
    return _then(_$Event_StateChangedImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      mode: null == mode
          ? _value.mode
          : mode // ignore: cast_nullable_to_non_nullable
              as CacheMode,
      state: null == state
          ? _value.state
          : state // ignore: cast_nullable_to_non_nullable
              as DownloadState,
      error: freezed == error
          ? _value.error
          : error // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc

class _$Event_StateChangedImpl extends Event_StateChanged {
  const _$Event_StateChangedImpl(
      {required this.id, required this.mode, required this.state, this.error})
      : super._();

  @override
  final String id;
  @override
  final CacheMode mode;
  @override
  final DownloadState state;
  @override
  final String? error;

  @override
  String toString() {
    return 'Event.stateChanged(id: $id, mode: $mode, state: $state, error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Event_StateChangedImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.mode, mode) || other.mode == mode) &&
            (identical(other.state, state) || other.state == state) &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, mode, state, error);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Event_StateChangedImplCopyWith<_$Event_StateChangedImpl> get copyWith =>
      __$$Event_StateChangedImplCopyWithImpl<_$Event_StateChangedImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)
        stateChanged,
    required TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)
        progress,
    required TResult Function(List<String> ids) removed,
    required TResult Function(String id, UploadState state, String? error)
        uploadStateChanged,
    required TResult Function(
            String id, bool hashing, int size, int done, int speed)
        uploadProgress,
    required TResult Function(List<String> ids) uploadRemoved,
  }) {
    return stateChanged(id, mode, state, error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult? Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult? Function(List<String> ids)? removed,
    TResult? Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult? Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult? Function(List<String> ids)? uploadRemoved,
  }) {
    return stateChanged?.call(id, mode, state, error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult Function(List<String> ids)? removed,
    TResult Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult Function(List<String> ids)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (stateChanged != null) {
      return stateChanged(id, mode, state, error);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Event_StateChanged value) stateChanged,
    required TResult Function(Event_Progress value) progress,
    required TResult Function(Event_Removed value) removed,
    required TResult Function(Event_UploadStateChanged value)
        uploadStateChanged,
    required TResult Function(Event_UploadProgress value) uploadProgress,
    required TResult Function(Event_UploadRemoved value) uploadRemoved,
  }) {
    return stateChanged(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Event_StateChanged value)? stateChanged,
    TResult? Function(Event_Progress value)? progress,
    TResult? Function(Event_Removed value)? removed,
    TResult? Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult? Function(Event_UploadProgress value)? uploadProgress,
    TResult? Function(Event_UploadRemoved value)? uploadRemoved,
  }) {
    return stateChanged?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Event_StateChanged value)? stateChanged,
    TResult Function(Event_Progress value)? progress,
    TResult Function(Event_Removed value)? removed,
    TResult Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult Function(Event_UploadProgress value)? uploadProgress,
    TResult Function(Event_UploadRemoved value)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (stateChanged != null) {
      return stateChanged(this);
    }
    return orElse();
  }
}

abstract class Event_StateChanged extends Event {
  const factory Event_StateChanged(
      {required final String id,
      required final CacheMode mode,
      required final DownloadState state,
      final String? error}) = _$Event_StateChangedImpl;
  const Event_StateChanged._() : super._();

  String get id;
  CacheMode get mode;
  DownloadState get state;
  String? get error;

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Event_StateChangedImplCopyWith<_$Event_StateChangedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Event_ProgressImplCopyWith<$Res> {
  factory _$$Event_ProgressImplCopyWith(_$Event_ProgressImpl value,
          $Res Function(_$Event_ProgressImpl) then) =
      __$$Event_ProgressImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String id, CacheMode mode, int? size, int downloaded, int speed});
}

/// @nodoc
class __$$Event_ProgressImplCopyWithImpl<$Res>
    extends _$EventCopyWithImpl<$Res, _$Event_ProgressImpl>
    implements _$$Event_ProgressImplCopyWith<$Res> {
  __$$Event_ProgressImplCopyWithImpl(
      _$Event_ProgressImpl _value, $Res Function(_$Event_ProgressImpl) _then)
      : super(_value, _then);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? mode = null,
    Object? size = freezed,
    Object? downloaded = null,
    Object? speed = null,
  }) {
    return _then(_$Event_ProgressImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      mode: null == mode
          ? _value.mode
          : mode // ignore: cast_nullable_to_non_nullable
              as CacheMode,
      size: freezed == size
          ? _value.size
          : size // ignore: cast_nullable_to_non_nullable
              as int?,
      downloaded: null == downloaded
          ? _value.downloaded
          : downloaded // ignore: cast_nullable_to_non_nullable
              as int,
      speed: null == speed
          ? _value.speed
          : speed // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$Event_ProgressImpl extends Event_Progress {
  const _$Event_ProgressImpl(
      {required this.id,
      required this.mode,
      this.size,
      required this.downloaded,
      required this.speed})
      : super._();

  @override
  final String id;
  @override
  final CacheMode mode;
  @override
  final int? size;
  @override
  final int downloaded;
  @override
  final int speed;

  @override
  String toString() {
    return 'Event.progress(id: $id, mode: $mode, size: $size, downloaded: $downloaded, speed: $speed)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Event_ProgressImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.mode, mode) || other.mode == mode) &&
            (identical(other.size, size) || other.size == size) &&
            (identical(other.downloaded, downloaded) ||
                other.downloaded == downloaded) &&
            (identical(other.speed, speed) || other.speed == speed));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, id, mode, size, downloaded, speed);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Event_ProgressImplCopyWith<_$Event_ProgressImpl> get copyWith =>
      __$$Event_ProgressImplCopyWithImpl<_$Event_ProgressImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)
        stateChanged,
    required TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)
        progress,
    required TResult Function(List<String> ids) removed,
    required TResult Function(String id, UploadState state, String? error)
        uploadStateChanged,
    required TResult Function(
            String id, bool hashing, int size, int done, int speed)
        uploadProgress,
    required TResult Function(List<String> ids) uploadRemoved,
  }) {
    return progress(id, mode, size, downloaded, speed);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult? Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult? Function(List<String> ids)? removed,
    TResult? Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult? Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult? Function(List<String> ids)? uploadRemoved,
  }) {
    return progress?.call(id, mode, size, downloaded, speed);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult Function(List<String> ids)? removed,
    TResult Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult Function(List<String> ids)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(id, mode, size, downloaded, speed);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Event_StateChanged value) stateChanged,
    required TResult Function(Event_Progress value) progress,
    required TResult Function(Event_Removed value) removed,
    required TResult Function(Event_UploadStateChanged value)
        uploadStateChanged,
    required TResult Function(Event_UploadProgress value) uploadProgress,
    required TResult Function(Event_UploadRemoved value) uploadRemoved,
  }) {
    return progress(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Event_StateChanged value)? stateChanged,
    TResult? Function(Event_Progress value)? progress,
    TResult? Function(Event_Removed value)? removed,
    TResult? Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult? Function(Event_UploadProgress value)? uploadProgress,
    TResult? Function(Event_UploadRemoved value)? uploadRemoved,
  }) {
    return progress?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Event_StateChanged value)? stateChanged,
    TResult Function(Event_Progress value)? progress,
    TResult Function(Event_Removed value)? removed,
    TResult Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult Function(Event_UploadProgress value)? uploadProgress,
    TResult Function(Event_UploadRemoved value)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(this);
    }
    return orElse();
  }
}

abstract class Event_Progress extends Event {
  const factory Event_Progress(
      {required final String id,
      required final CacheMode mode,
      final int? size,
      required final int downloaded,
      required final int speed}) = _$Event_ProgressImpl;
  const Event_Progress._() : super._();

  String get id;
  CacheMode get mode;
  int? get size;
  int get downloaded;
  int get speed;

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Event_ProgressImplCopyWith<_$Event_ProgressImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Event_RemovedImplCopyWith<$Res> {
  factory _$$Event_RemovedImplCopyWith(
          _$Event_RemovedImpl value, $Res Function(_$Event_RemovedImpl) then) =
      __$$Event_RemovedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<String> ids});
}

/// @nodoc
class __$$Event_RemovedImplCopyWithImpl<$Res>
    extends _$EventCopyWithImpl<$Res, _$Event_RemovedImpl>
    implements _$$Event_RemovedImplCopyWith<$Res> {
  __$$Event_RemovedImplCopyWithImpl(
      _$Event_RemovedImpl _value, $Res Function(_$Event_RemovedImpl) _then)
      : super(_value, _then);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? ids = null,
  }) {
    return _then(_$Event_RemovedImpl(
      ids: null == ids
          ? _value._ids
          : ids // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc

class _$Event_RemovedImpl extends Event_Removed {
  const _$Event_RemovedImpl({required final List<String> ids})
      : _ids = ids,
        super._();

  final List<String> _ids;
  @override
  List<String> get ids {
    if (_ids is EqualUnmodifiableListView) return _ids;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_ids);
  }

  @override
  String toString() {
    return 'Event.removed(ids: $ids)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Event_RemovedImpl &&
            const DeepCollectionEquality().equals(other._ids, _ids));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_ids));

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Event_RemovedImplCopyWith<_$Event_RemovedImpl> get copyWith =>
      __$$Event_RemovedImplCopyWithImpl<_$Event_RemovedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)
        stateChanged,
    required TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)
        progress,
    required TResult Function(List<String> ids) removed,
    required TResult Function(String id, UploadState state, String? error)
        uploadStateChanged,
    required TResult Function(
            String id, bool hashing, int size, int done, int speed)
        uploadProgress,
    required TResult Function(List<String> ids) uploadRemoved,
  }) {
    return removed(ids);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult? Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult? Function(List<String> ids)? removed,
    TResult? Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult? Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult? Function(List<String> ids)? uploadRemoved,
  }) {
    return removed?.call(ids);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult Function(List<String> ids)? removed,
    TResult Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult Function(List<String> ids)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (removed != null) {
      return removed(ids);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Event_StateChanged value) stateChanged,
    required TResult Function(Event_Progress value) progress,
    required TResult Function(Event_Removed value) removed,
    required TResult Function(Event_UploadStateChanged value)
        uploadStateChanged,
    required TResult Function(Event_UploadProgress value) uploadProgress,
    required TResult Function(Event_UploadRemoved value) uploadRemoved,
  }) {
    return removed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Event_StateChanged value)? stateChanged,
    TResult? Function(Event_Progress value)? progress,
    TResult? Function(Event_Removed value)? removed,
    TResult? Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult? Function(Event_UploadProgress value)? uploadProgress,
    TResult? Function(Event_UploadRemoved value)? uploadRemoved,
  }) {
    return removed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Event_StateChanged value)? stateChanged,
    TResult Function(Event_Progress value)? progress,
    TResult Function(Event_Removed value)? removed,
    TResult Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult Function(Event_UploadProgress value)? uploadProgress,
    TResult Function(Event_UploadRemoved value)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (removed != null) {
      return removed(this);
    }
    return orElse();
  }
}

abstract class Event_Removed extends Event {
  const factory Event_Removed({required final List<String> ids}) =
      _$Event_RemovedImpl;
  const Event_Removed._() : super._();

  List<String> get ids;

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Event_RemovedImplCopyWith<_$Event_RemovedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Event_UploadStateChangedImplCopyWith<$Res> {
  factory _$$Event_UploadStateChangedImplCopyWith(
          _$Event_UploadStateChangedImpl value,
          $Res Function(_$Event_UploadStateChangedImpl) then) =
      __$$Event_UploadStateChangedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String id, UploadState state, String? error});
}

/// @nodoc
class __$$Event_UploadStateChangedImplCopyWithImpl<$Res>
    extends _$EventCopyWithImpl<$Res, _$Event_UploadStateChangedImpl>
    implements _$$Event_UploadStateChangedImplCopyWith<$Res> {
  __$$Event_UploadStateChangedImplCopyWithImpl(
      _$Event_UploadStateChangedImpl _value,
      $Res Function(_$Event_UploadStateChangedImpl) _then)
      : super(_value, _then);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? state = null,
    Object? error = freezed,
  }) {
    return _then(_$Event_UploadStateChangedImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      state: null == state
          ? _value.state
          : state // ignore: cast_nullable_to_non_nullable
              as UploadState,
      error: freezed == error
          ? _value.error
          : error // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc

class _$Event_UploadStateChangedImpl extends Event_UploadStateChanged {
  const _$Event_UploadStateChangedImpl(
      {required this.id, required this.state, this.error})
      : super._();

  @override
  final String id;
  @override
  final UploadState state;
  @override
  final String? error;

  @override
  String toString() {
    return 'Event.uploadStateChanged(id: $id, state: $state, error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Event_UploadStateChangedImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.state, state) || other.state == state) &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, state, error);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Event_UploadStateChangedImplCopyWith<_$Event_UploadStateChangedImpl>
      get copyWith => __$$Event_UploadStateChangedImplCopyWithImpl<
          _$Event_UploadStateChangedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)
        stateChanged,
    required TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)
        progress,
    required TResult Function(List<String> ids) removed,
    required TResult Function(String id, UploadState state, String? error)
        uploadStateChanged,
    required TResult Function(
            String id, bool hashing, int size, int done, int speed)
        uploadProgress,
    required TResult Function(List<String> ids) uploadRemoved,
  }) {
    return uploadStateChanged(id, state, error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult? Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult? Function(List<String> ids)? removed,
    TResult? Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult? Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult? Function(List<String> ids)? uploadRemoved,
  }) {
    return uploadStateChanged?.call(id, state, error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult Function(List<String> ids)? removed,
    TResult Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult Function(List<String> ids)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (uploadStateChanged != null) {
      return uploadStateChanged(id, state, error);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Event_StateChanged value) stateChanged,
    required TResult Function(Event_Progress value) progress,
    required TResult Function(Event_Removed value) removed,
    required TResult Function(Event_UploadStateChanged value)
        uploadStateChanged,
    required TResult Function(Event_UploadProgress value) uploadProgress,
    required TResult Function(Event_UploadRemoved value) uploadRemoved,
  }) {
    return uploadStateChanged(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Event_StateChanged value)? stateChanged,
    TResult? Function(Event_Progress value)? progress,
    TResult? Function(Event_Removed value)? removed,
    TResult? Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult? Function(Event_UploadProgress value)? uploadProgress,
    TResult? Function(Event_UploadRemoved value)? uploadRemoved,
  }) {
    return uploadStateChanged?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Event_StateChanged value)? stateChanged,
    TResult Function(Event_Progress value)? progress,
    TResult Function(Event_Removed value)? removed,
    TResult Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult Function(Event_UploadProgress value)? uploadProgress,
    TResult Function(Event_UploadRemoved value)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (uploadStateChanged != null) {
      return uploadStateChanged(this);
    }
    return orElse();
  }
}

abstract class Event_UploadStateChanged extends Event {
  const factory Event_UploadStateChanged(
      {required final String id,
      required final UploadState state,
      final String? error}) = _$Event_UploadStateChangedImpl;
  const Event_UploadStateChanged._() : super._();

  String get id;
  UploadState get state;
  String? get error;

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Event_UploadStateChangedImplCopyWith<_$Event_UploadStateChangedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Event_UploadProgressImplCopyWith<$Res> {
  factory _$$Event_UploadProgressImplCopyWith(_$Event_UploadProgressImpl value,
          $Res Function(_$Event_UploadProgressImpl) then) =
      __$$Event_UploadProgressImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String id, bool hashing, int size, int done, int speed});
}

/// @nodoc
class __$$Event_UploadProgressImplCopyWithImpl<$Res>
    extends _$EventCopyWithImpl<$Res, _$Event_UploadProgressImpl>
    implements _$$Event_UploadProgressImplCopyWith<$Res> {
  __$$Event_UploadProgressImplCopyWithImpl(_$Event_UploadProgressImpl _value,
      $Res Function(_$Event_UploadProgressImpl) _then)
      : super(_value, _then);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? hashing = null,
    Object? size = null,
    Object? done = null,
    Object? speed = null,
  }) {
    return _then(_$Event_UploadProgressImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      hashing: null == hashing
          ? _value.hashing
          : hashing // ignore: cast_nullable_to_non_nullable
              as bool,
      size: null == size
          ? _value.size
          : size // ignore: cast_nullable_to_non_nullable
              as int,
      done: null == done
          ? _value.done
          : done // ignore: cast_nullable_to_non_nullable
              as int,
      speed: null == speed
          ? _value.speed
          : speed // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$Event_UploadProgressImpl extends Event_UploadProgress {
  const _$Event_UploadProgressImpl(
      {required this.id,
      required this.hashing,
      required this.size,
      required this.done,
      required this.speed})
      : super._();

  @override
  final String id;
  @override
  final bool hashing;
  @override
  final int size;
  @override
  final int done;
  @override
  final int speed;

  @override
  String toString() {
    return 'Event.uploadProgress(id: $id, hashing: $hashing, size: $size, done: $done, speed: $speed)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Event_UploadProgressImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.hashing, hashing) || other.hashing == hashing) &&
            (identical(other.size, size) || other.size == size) &&
            (identical(other.done, done) || other.done == done) &&
            (identical(other.speed, speed) || other.speed == speed));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, hashing, size, done, speed);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Event_UploadProgressImplCopyWith<_$Event_UploadProgressImpl>
      get copyWith =>
          __$$Event_UploadProgressImplCopyWithImpl<_$Event_UploadProgressImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)
        stateChanged,
    required TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)
        progress,
    required TResult Function(List<String> ids) removed,
    required TResult Function(String id, UploadState state, String? error)
        uploadStateChanged,
    required TResult Function(
            String id, bool hashing, int size, int done, int speed)
        uploadProgress,
    required TResult Function(List<String> ids) uploadRemoved,
  }) {
    return uploadProgress(id, hashing, size, done, speed);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult? Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult? Function(List<String> ids)? removed,
    TResult? Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult? Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult? Function(List<String> ids)? uploadRemoved,
  }) {
    return uploadProgress?.call(id, hashing, size, done, speed);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult Function(List<String> ids)? removed,
    TResult Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult Function(List<String> ids)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (uploadProgress != null) {
      return uploadProgress(id, hashing, size, done, speed);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Event_StateChanged value) stateChanged,
    required TResult Function(Event_Progress value) progress,
    required TResult Function(Event_Removed value) removed,
    required TResult Function(Event_UploadStateChanged value)
        uploadStateChanged,
    required TResult Function(Event_UploadProgress value) uploadProgress,
    required TResult Function(Event_UploadRemoved value) uploadRemoved,
  }) {
    return uploadProgress(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Event_StateChanged value)? stateChanged,
    TResult? Function(Event_Progress value)? progress,
    TResult? Function(Event_Removed value)? removed,
    TResult? Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult? Function(Event_UploadProgress value)? uploadProgress,
    TResult? Function(Event_UploadRemoved value)? uploadRemoved,
  }) {
    return uploadProgress?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Event_StateChanged value)? stateChanged,
    TResult Function(Event_Progress value)? progress,
    TResult Function(Event_Removed value)? removed,
    TResult Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult Function(Event_UploadProgress value)? uploadProgress,
    TResult Function(Event_UploadRemoved value)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (uploadProgress != null) {
      return uploadProgress(this);
    }
    return orElse();
  }
}

abstract class Event_UploadProgress extends Event {
  const factory Event_UploadProgress(
      {required final String id,
      required final bool hashing,
      required final int size,
      required final int done,
      required final int speed}) = _$Event_UploadProgressImpl;
  const Event_UploadProgress._() : super._();

  String get id;
  bool get hashing;
  int get size;
  int get done;
  int get speed;

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Event_UploadProgressImplCopyWith<_$Event_UploadProgressImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Event_UploadRemovedImplCopyWith<$Res> {
  factory _$$Event_UploadRemovedImplCopyWith(_$Event_UploadRemovedImpl value,
          $Res Function(_$Event_UploadRemovedImpl) then) =
      __$$Event_UploadRemovedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<String> ids});
}

/// @nodoc
class __$$Event_UploadRemovedImplCopyWithImpl<$Res>
    extends _$EventCopyWithImpl<$Res, _$Event_UploadRemovedImpl>
    implements _$$Event_UploadRemovedImplCopyWith<$Res> {
  __$$Event_UploadRemovedImplCopyWithImpl(_$Event_UploadRemovedImpl _value,
      $Res Function(_$Event_UploadRemovedImpl) _then)
      : super(_value, _then);

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? ids = null,
  }) {
    return _then(_$Event_UploadRemovedImpl(
      ids: null == ids
          ? _value._ids
          : ids // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc

class _$Event_UploadRemovedImpl extends Event_UploadRemoved {
  const _$Event_UploadRemovedImpl({required final List<String> ids})
      : _ids = ids,
        super._();

  final List<String> _ids;
  @override
  List<String> get ids {
    if (_ids is EqualUnmodifiableListView) return _ids;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_ids);
  }

  @override
  String toString() {
    return 'Event.uploadRemoved(ids: $ids)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Event_UploadRemovedImpl &&
            const DeepCollectionEquality().equals(other._ids, _ids));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_ids));

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Event_UploadRemovedImplCopyWith<_$Event_UploadRemovedImpl> get copyWith =>
      __$$Event_UploadRemovedImplCopyWithImpl<_$Event_UploadRemovedImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)
        stateChanged,
    required TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)
        progress,
    required TResult Function(List<String> ids) removed,
    required TResult Function(String id, UploadState state, String? error)
        uploadStateChanged,
    required TResult Function(
            String id, bool hashing, int size, int done, int speed)
        uploadProgress,
    required TResult Function(List<String> ids) uploadRemoved,
  }) {
    return uploadRemoved(ids);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult? Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult? Function(List<String> ids)? removed,
    TResult? Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult? Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult? Function(List<String> ids)? uploadRemoved,
  }) {
    return uploadRemoved?.call(ids);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String id, CacheMode mode, DownloadState state, String? error)?
        stateChanged,
    TResult Function(
            String id, CacheMode mode, int? size, int downloaded, int speed)?
        progress,
    TResult Function(List<String> ids)? removed,
    TResult Function(String id, UploadState state, String? error)?
        uploadStateChanged,
    TResult Function(String id, bool hashing, int size, int done, int speed)?
        uploadProgress,
    TResult Function(List<String> ids)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (uploadRemoved != null) {
      return uploadRemoved(ids);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Event_StateChanged value) stateChanged,
    required TResult Function(Event_Progress value) progress,
    required TResult Function(Event_Removed value) removed,
    required TResult Function(Event_UploadStateChanged value)
        uploadStateChanged,
    required TResult Function(Event_UploadProgress value) uploadProgress,
    required TResult Function(Event_UploadRemoved value) uploadRemoved,
  }) {
    return uploadRemoved(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Event_StateChanged value)? stateChanged,
    TResult? Function(Event_Progress value)? progress,
    TResult? Function(Event_Removed value)? removed,
    TResult? Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult? Function(Event_UploadProgress value)? uploadProgress,
    TResult? Function(Event_UploadRemoved value)? uploadRemoved,
  }) {
    return uploadRemoved?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Event_StateChanged value)? stateChanged,
    TResult Function(Event_Progress value)? progress,
    TResult Function(Event_Removed value)? removed,
    TResult Function(Event_UploadStateChanged value)? uploadStateChanged,
    TResult Function(Event_UploadProgress value)? uploadProgress,
    TResult Function(Event_UploadRemoved value)? uploadRemoved,
    required TResult orElse(),
  }) {
    if (uploadRemoved != null) {
      return uploadRemoved(this);
    }
    return orElse();
  }
}

abstract class Event_UploadRemoved extends Event {
  const factory Event_UploadRemoved({required final List<String> ids}) =
      _$Event_UploadRemovedImpl;
  const Event_UploadRemoved._() : super._();

  List<String> get ids;

  /// Create a copy of Event
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Event_UploadRemovedImplCopyWith<_$Event_UploadRemovedImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$FetchEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int? size, int downloaded) progress,
    required TResult Function(String path) success,
    required TResult Function(SdkError error) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int? size, int downloaded)? progress,
    TResult? Function(String path)? success,
    TResult? Function(SdkError error)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int? size, int downloaded)? progress,
    TResult Function(String path)? success,
    TResult Function(SdkError error)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FetchEvent_Progress value) progress,
    required TResult Function(FetchEvent_Success value) success,
    required TResult Function(FetchEvent_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FetchEvent_Progress value)? progress,
    TResult? Function(FetchEvent_Success value)? success,
    TResult? Function(FetchEvent_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FetchEvent_Progress value)? progress,
    TResult Function(FetchEvent_Success value)? success,
    TResult Function(FetchEvent_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FetchEventCopyWith<$Res> {
  factory $FetchEventCopyWith(
          FetchEvent value, $Res Function(FetchEvent) then) =
      _$FetchEventCopyWithImpl<$Res, FetchEvent>;
}

/// @nodoc
class _$FetchEventCopyWithImpl<$Res, $Val extends FetchEvent>
    implements $FetchEventCopyWith<$Res> {
  _$FetchEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$FetchEvent_ProgressImplCopyWith<$Res> {
  factory _$$FetchEvent_ProgressImplCopyWith(_$FetchEvent_ProgressImpl value,
          $Res Function(_$FetchEvent_ProgressImpl) then) =
      __$$FetchEvent_ProgressImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int? size, int downloaded});
}

/// @nodoc
class __$$FetchEvent_ProgressImplCopyWithImpl<$Res>
    extends _$FetchEventCopyWithImpl<$Res, _$FetchEvent_ProgressImpl>
    implements _$$FetchEvent_ProgressImplCopyWith<$Res> {
  __$$FetchEvent_ProgressImplCopyWithImpl(_$FetchEvent_ProgressImpl _value,
      $Res Function(_$FetchEvent_ProgressImpl) _then)
      : super(_value, _then);

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? size = freezed,
    Object? downloaded = null,
  }) {
    return _then(_$FetchEvent_ProgressImpl(
      size: freezed == size
          ? _value.size
          : size // ignore: cast_nullable_to_non_nullable
              as int?,
      downloaded: null == downloaded
          ? _value.downloaded
          : downloaded // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$FetchEvent_ProgressImpl extends FetchEvent_Progress {
  const _$FetchEvent_ProgressImpl({this.size, required this.downloaded})
      : super._();

  @override
  final int? size;
  @override
  final int downloaded;

  @override
  String toString() {
    return 'FetchEvent.progress(size: $size, downloaded: $downloaded)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FetchEvent_ProgressImpl &&
            (identical(other.size, size) || other.size == size) &&
            (identical(other.downloaded, downloaded) ||
                other.downloaded == downloaded));
  }

  @override
  int get hashCode => Object.hash(runtimeType, size, downloaded);

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FetchEvent_ProgressImplCopyWith<_$FetchEvent_ProgressImpl> get copyWith =>
      __$$FetchEvent_ProgressImplCopyWithImpl<_$FetchEvent_ProgressImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int? size, int downloaded) progress,
    required TResult Function(String path) success,
    required TResult Function(SdkError error) error,
  }) {
    return progress(size, downloaded);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int? size, int downloaded)? progress,
    TResult? Function(String path)? success,
    TResult? Function(SdkError error)? error,
  }) {
    return progress?.call(size, downloaded);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int? size, int downloaded)? progress,
    TResult Function(String path)? success,
    TResult Function(SdkError error)? error,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(size, downloaded);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FetchEvent_Progress value) progress,
    required TResult Function(FetchEvent_Success value) success,
    required TResult Function(FetchEvent_Error value) error,
  }) {
    return progress(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FetchEvent_Progress value)? progress,
    TResult? Function(FetchEvent_Success value)? success,
    TResult? Function(FetchEvent_Error value)? error,
  }) {
    return progress?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FetchEvent_Progress value)? progress,
    TResult Function(FetchEvent_Success value)? success,
    TResult Function(FetchEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(this);
    }
    return orElse();
  }
}

abstract class FetchEvent_Progress extends FetchEvent {
  const factory FetchEvent_Progress(
      {final int? size,
      required final int downloaded}) = _$FetchEvent_ProgressImpl;
  const FetchEvent_Progress._() : super._();

  int? get size;
  int get downloaded;

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FetchEvent_ProgressImplCopyWith<_$FetchEvent_ProgressImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FetchEvent_SuccessImplCopyWith<$Res> {
  factory _$$FetchEvent_SuccessImplCopyWith(_$FetchEvent_SuccessImpl value,
          $Res Function(_$FetchEvent_SuccessImpl) then) =
      __$$FetchEvent_SuccessImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String path});
}

/// @nodoc
class __$$FetchEvent_SuccessImplCopyWithImpl<$Res>
    extends _$FetchEventCopyWithImpl<$Res, _$FetchEvent_SuccessImpl>
    implements _$$FetchEvent_SuccessImplCopyWith<$Res> {
  __$$FetchEvent_SuccessImplCopyWithImpl(_$FetchEvent_SuccessImpl _value,
      $Res Function(_$FetchEvent_SuccessImpl) _then)
      : super(_value, _then);

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? path = null,
  }) {
    return _then(_$FetchEvent_SuccessImpl(
      path: null == path
          ? _value.path
          : path // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$FetchEvent_SuccessImpl extends FetchEvent_Success {
  const _$FetchEvent_SuccessImpl({required this.path}) : super._();

  @override
  final String path;

  @override
  String toString() {
    return 'FetchEvent.success(path: $path)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FetchEvent_SuccessImpl &&
            (identical(other.path, path) || other.path == path));
  }

  @override
  int get hashCode => Object.hash(runtimeType, path);

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FetchEvent_SuccessImplCopyWith<_$FetchEvent_SuccessImpl> get copyWith =>
      __$$FetchEvent_SuccessImplCopyWithImpl<_$FetchEvent_SuccessImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int? size, int downloaded) progress,
    required TResult Function(String path) success,
    required TResult Function(SdkError error) error,
  }) {
    return success(path);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int? size, int downloaded)? progress,
    TResult? Function(String path)? success,
    TResult? Function(SdkError error)? error,
  }) {
    return success?.call(path);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int? size, int downloaded)? progress,
    TResult Function(String path)? success,
    TResult Function(SdkError error)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(path);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FetchEvent_Progress value) progress,
    required TResult Function(FetchEvent_Success value) success,
    required TResult Function(FetchEvent_Error value) error,
  }) {
    return success(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FetchEvent_Progress value)? progress,
    TResult? Function(FetchEvent_Success value)? success,
    TResult? Function(FetchEvent_Error value)? error,
  }) {
    return success?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FetchEvent_Progress value)? progress,
    TResult Function(FetchEvent_Success value)? success,
    TResult Function(FetchEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (success != null) {
      return success(this);
    }
    return orElse();
  }
}

abstract class FetchEvent_Success extends FetchEvent {
  const factory FetchEvent_Success({required final String path}) =
      _$FetchEvent_SuccessImpl;
  const FetchEvent_Success._() : super._();

  String get path;

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FetchEvent_SuccessImplCopyWith<_$FetchEvent_SuccessImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$FetchEvent_ErrorImplCopyWith<$Res> {
  factory _$$FetchEvent_ErrorImplCopyWith(_$FetchEvent_ErrorImpl value,
          $Res Function(_$FetchEvent_ErrorImpl) then) =
      __$$FetchEvent_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({SdkError error});

  $SdkErrorCopyWith<$Res> get error;
}

/// @nodoc
class __$$FetchEvent_ErrorImplCopyWithImpl<$Res>
    extends _$FetchEventCopyWithImpl<$Res, _$FetchEvent_ErrorImpl>
    implements _$$FetchEvent_ErrorImplCopyWith<$Res> {
  __$$FetchEvent_ErrorImplCopyWithImpl(_$FetchEvent_ErrorImpl _value,
      $Res Function(_$FetchEvent_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? error = null,
  }) {
    return _then(_$FetchEvent_ErrorImpl(
      error: null == error
          ? _value.error
          : error // ignore: cast_nullable_to_non_nullable
              as SdkError,
    ));
  }

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $SdkErrorCopyWith<$Res> get error {
    return $SdkErrorCopyWith<$Res>(_value.error, (value) {
      return _then(_value.copyWith(error: value));
    });
  }
}

/// @nodoc

class _$FetchEvent_ErrorImpl extends FetchEvent_Error {
  const _$FetchEvent_ErrorImpl({required this.error}) : super._();

  @override
  final SdkError error;

  @override
  String toString() {
    return 'FetchEvent.error(error: $error)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FetchEvent_ErrorImpl &&
            (identical(other.error, error) || other.error == error));
  }

  @override
  int get hashCode => Object.hash(runtimeType, error);

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FetchEvent_ErrorImplCopyWith<_$FetchEvent_ErrorImpl> get copyWith =>
      __$$FetchEvent_ErrorImplCopyWithImpl<_$FetchEvent_ErrorImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int? size, int downloaded) progress,
    required TResult Function(String path) success,
    required TResult Function(SdkError error) error,
  }) {
    return error(this.error);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int? size, int downloaded)? progress,
    TResult? Function(String path)? success,
    TResult? Function(SdkError error)? error,
  }) {
    return error?.call(this.error);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int? size, int downloaded)? progress,
    TResult Function(String path)? success,
    TResult Function(SdkError error)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this.error);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(FetchEvent_Progress value) progress,
    required TResult Function(FetchEvent_Success value) success,
    required TResult Function(FetchEvent_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(FetchEvent_Progress value)? progress,
    TResult? Function(FetchEvent_Success value)? success,
    TResult? Function(FetchEvent_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(FetchEvent_Progress value)? progress,
    TResult Function(FetchEvent_Success value)? success,
    TResult Function(FetchEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class FetchEvent_Error extends FetchEvent {
  const factory FetchEvent_Error({required final SdkError error}) =
      _$FetchEvent_ErrorImpl;
  const FetchEvent_Error._() : super._();

  SdkError get error;

  /// Create a copy of FetchEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FetchEvent_ErrorImplCopyWith<_$FetchEvent_ErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$SdkError {
  String get msg => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SdkErrorCopyWith<SdkError> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SdkErrorCopyWith<$Res> {
  factory $SdkErrorCopyWith(SdkError value, $Res Function(SdkError) then) =
      _$SdkErrorCopyWithImpl<$Res, SdkError>;
  @useResult
  $Res call({String msg});
}

/// @nodoc
class _$SdkErrorCopyWithImpl<$Res, $Val extends SdkError>
    implements $SdkErrorCopyWith<$Res> {
  _$SdkErrorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_value.copyWith(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SdkError_UrlExpiredImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_UrlExpiredImplCopyWith(_$SdkError_UrlExpiredImpl value,
          $Res Function(_$SdkError_UrlExpiredImpl) then) =
      __$$SdkError_UrlExpiredImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({int status, String msg});
}

/// @nodoc
class __$$SdkError_UrlExpiredImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_UrlExpiredImpl>
    implements _$$SdkError_UrlExpiredImplCopyWith<$Res> {
  __$$SdkError_UrlExpiredImplCopyWithImpl(_$SdkError_UrlExpiredImpl _value,
      $Res Function(_$SdkError_UrlExpiredImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? status = null,
    Object? msg = null,
  }) {
    return _then(_$SdkError_UrlExpiredImpl(
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as int,
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_UrlExpiredImpl extends SdkError_UrlExpired {
  const _$SdkError_UrlExpiredImpl({required this.status, required this.msg})
      : super._();

  @override
  final int status;
  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.urlExpired(status: $status, msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_UrlExpiredImpl &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, status, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_UrlExpiredImplCopyWith<_$SdkError_UrlExpiredImpl> get copyWith =>
      __$$SdkError_UrlExpiredImplCopyWithImpl<_$SdkError_UrlExpiredImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return urlExpired(status, msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return urlExpired?.call(status, msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (urlExpired != null) {
      return urlExpired(status, msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return urlExpired(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return urlExpired?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (urlExpired != null) {
      return urlExpired(this);
    }
    return orElse();
  }
}

abstract class SdkError_UrlExpired extends SdkError {
  const factory SdkError_UrlExpired(
      {required final int status,
      required final String msg}) = _$SdkError_UrlExpiredImpl;
  const SdkError_UrlExpired._() : super._();

  int get status;
  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_UrlExpiredImplCopyWith<_$SdkError_UrlExpiredImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_NotFoundImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_NotFoundImplCopyWith(_$SdkError_NotFoundImpl value,
          $Res Function(_$SdkError_NotFoundImpl) then) =
      __$$SdkError_NotFoundImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_NotFoundImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_NotFoundImpl>
    implements _$$SdkError_NotFoundImplCopyWith<$Res> {
  __$$SdkError_NotFoundImplCopyWithImpl(_$SdkError_NotFoundImpl _value,
      $Res Function(_$SdkError_NotFoundImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_NotFoundImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_NotFoundImpl extends SdkError_NotFound {
  const _$SdkError_NotFoundImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.notFound(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_NotFoundImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_NotFoundImplCopyWith<_$SdkError_NotFoundImpl> get copyWith =>
      __$$SdkError_NotFoundImplCopyWithImpl<_$SdkError_NotFoundImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return notFound(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return notFound?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (notFound != null) {
      return notFound(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return notFound(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return notFound?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (notFound != null) {
      return notFound(this);
    }
    return orElse();
  }
}

abstract class SdkError_NotFound extends SdkError {
  const factory SdkError_NotFound({required final String msg}) =
      _$SdkError_NotFoundImpl;
  const SdkError_NotFound._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_NotFoundImplCopyWith<_$SdkError_NotFoundImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_HttpImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_HttpImplCopyWith(
          _$SdkError_HttpImpl value, $Res Function(_$SdkError_HttpImpl) then) =
      __$$SdkError_HttpImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({int status, String msg});
}

/// @nodoc
class __$$SdkError_HttpImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_HttpImpl>
    implements _$$SdkError_HttpImplCopyWith<$Res> {
  __$$SdkError_HttpImplCopyWithImpl(
      _$SdkError_HttpImpl _value, $Res Function(_$SdkError_HttpImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? status = null,
    Object? msg = null,
  }) {
    return _then(_$SdkError_HttpImpl(
      status: null == status
          ? _value.status
          : status // ignore: cast_nullable_to_non_nullable
              as int,
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_HttpImpl extends SdkError_Http {
  const _$SdkError_HttpImpl({required this.status, required this.msg})
      : super._();

  @override
  final int status;
  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.http(status: $status, msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_HttpImpl &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, status, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_HttpImplCopyWith<_$SdkError_HttpImpl> get copyWith =>
      __$$SdkError_HttpImplCopyWithImpl<_$SdkError_HttpImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return http(status, msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return http?.call(status, msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (http != null) {
      return http(status, msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return http(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return http?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (http != null) {
      return http(this);
    }
    return orElse();
  }
}

abstract class SdkError_Http extends SdkError {
  const factory SdkError_Http(
      {required final int status,
      required final String msg}) = _$SdkError_HttpImpl;
  const SdkError_Http._() : super._();

  int get status;
  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_HttpImplCopyWith<_$SdkError_HttpImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_ServerImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_ServerImplCopyWith(_$SdkError_ServerImpl value,
          $Res Function(_$SdkError_ServerImpl) then) =
      __$$SdkError_ServerImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_ServerImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_ServerImpl>
    implements _$$SdkError_ServerImplCopyWith<$Res> {
  __$$SdkError_ServerImplCopyWithImpl(
      _$SdkError_ServerImpl _value, $Res Function(_$SdkError_ServerImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_ServerImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_ServerImpl extends SdkError_Server {
  const _$SdkError_ServerImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.server(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_ServerImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_ServerImplCopyWith<_$SdkError_ServerImpl> get copyWith =>
      __$$SdkError_ServerImplCopyWithImpl<_$SdkError_ServerImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return server(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return server?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (server != null) {
      return server(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return server(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return server?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (server != null) {
      return server(this);
    }
    return orElse();
  }
}

abstract class SdkError_Server extends SdkError {
  const factory SdkError_Server({required final String msg}) =
      _$SdkError_ServerImpl;
  const SdkError_Server._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_ServerImplCopyWith<_$SdkError_ServerImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_SourceImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_SourceImplCopyWith(_$SdkError_SourceImpl value,
          $Res Function(_$SdkError_SourceImpl) then) =
      __$$SdkError_SourceImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_SourceImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_SourceImpl>
    implements _$$SdkError_SourceImplCopyWith<$Res> {
  __$$SdkError_SourceImplCopyWithImpl(
      _$SdkError_SourceImpl _value, $Res Function(_$SdkError_SourceImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_SourceImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_SourceImpl extends SdkError_Source {
  const _$SdkError_SourceImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.source(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_SourceImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_SourceImplCopyWith<_$SdkError_SourceImpl> get copyWith =>
      __$$SdkError_SourceImplCopyWithImpl<_$SdkError_SourceImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return source(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return source?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (source != null) {
      return source(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return source(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return source?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (source != null) {
      return source(this);
    }
    return orElse();
  }
}

abstract class SdkError_Source extends SdkError {
  const factory SdkError_Source({required final String msg}) =
      _$SdkError_SourceImpl;
  const SdkError_Source._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_SourceImplCopyWith<_$SdkError_SourceImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_NetworkImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_NetworkImplCopyWith(_$SdkError_NetworkImpl value,
          $Res Function(_$SdkError_NetworkImpl) then) =
      __$$SdkError_NetworkImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_NetworkImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_NetworkImpl>
    implements _$$SdkError_NetworkImplCopyWith<$Res> {
  __$$SdkError_NetworkImplCopyWithImpl(_$SdkError_NetworkImpl _value,
      $Res Function(_$SdkError_NetworkImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_NetworkImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_NetworkImpl extends SdkError_Network {
  const _$SdkError_NetworkImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.network(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_NetworkImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_NetworkImplCopyWith<_$SdkError_NetworkImpl> get copyWith =>
      __$$SdkError_NetworkImplCopyWithImpl<_$SdkError_NetworkImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return network(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return network?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (network != null) {
      return network(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return network(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return network?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (network != null) {
      return network(this);
    }
    return orElse();
  }
}

abstract class SdkError_Network extends SdkError {
  const factory SdkError_Network({required final String msg}) =
      _$SdkError_NetworkImpl;
  const SdkError_Network._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_NetworkImplCopyWith<_$SdkError_NetworkImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_IoImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_IoImplCopyWith(
          _$SdkError_IoImpl value, $Res Function(_$SdkError_IoImpl) then) =
      __$$SdkError_IoImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_IoImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_IoImpl>
    implements _$$SdkError_IoImplCopyWith<$Res> {
  __$$SdkError_IoImplCopyWithImpl(
      _$SdkError_IoImpl _value, $Res Function(_$SdkError_IoImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_IoImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_IoImpl extends SdkError_Io {
  const _$SdkError_IoImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.io(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_IoImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_IoImplCopyWith<_$SdkError_IoImpl> get copyWith =>
      __$$SdkError_IoImplCopyWithImpl<_$SdkError_IoImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return io(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return io?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (io != null) {
      return io(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return io(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return io?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (io != null) {
      return io(this);
    }
    return orElse();
  }
}

abstract class SdkError_Io extends SdkError {
  const factory SdkError_Io({required final String msg}) = _$SdkError_IoImpl;
  const SdkError_Io._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_IoImplCopyWith<_$SdkError_IoImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_Md5MismatchImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_Md5MismatchImplCopyWith(_$SdkError_Md5MismatchImpl value,
          $Res Function(_$SdkError_Md5MismatchImpl) then) =
      __$$SdkError_Md5MismatchImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_Md5MismatchImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_Md5MismatchImpl>
    implements _$$SdkError_Md5MismatchImplCopyWith<$Res> {
  __$$SdkError_Md5MismatchImplCopyWithImpl(_$SdkError_Md5MismatchImpl _value,
      $Res Function(_$SdkError_Md5MismatchImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_Md5MismatchImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_Md5MismatchImpl extends SdkError_Md5Mismatch {
  const _$SdkError_Md5MismatchImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.md5Mismatch(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_Md5MismatchImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_Md5MismatchImplCopyWith<_$SdkError_Md5MismatchImpl>
      get copyWith =>
          __$$SdkError_Md5MismatchImplCopyWithImpl<_$SdkError_Md5MismatchImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return md5Mismatch(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return md5Mismatch?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (md5Mismatch != null) {
      return md5Mismatch(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return md5Mismatch(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return md5Mismatch?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (md5Mismatch != null) {
      return md5Mismatch(this);
    }
    return orElse();
  }
}

abstract class SdkError_Md5Mismatch extends SdkError {
  const factory SdkError_Md5Mismatch({required final String msg}) =
      _$SdkError_Md5MismatchImpl;
  const SdkError_Md5Mismatch._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_Md5MismatchImplCopyWith<_$SdkError_Md5MismatchImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_InvalidUrlImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_InvalidUrlImplCopyWith(_$SdkError_InvalidUrlImpl value,
          $Res Function(_$SdkError_InvalidUrlImpl) then) =
      __$$SdkError_InvalidUrlImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_InvalidUrlImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_InvalidUrlImpl>
    implements _$$SdkError_InvalidUrlImplCopyWith<$Res> {
  __$$SdkError_InvalidUrlImplCopyWithImpl(_$SdkError_InvalidUrlImpl _value,
      $Res Function(_$SdkError_InvalidUrlImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_InvalidUrlImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_InvalidUrlImpl extends SdkError_InvalidUrl {
  const _$SdkError_InvalidUrlImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.invalidUrl(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_InvalidUrlImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_InvalidUrlImplCopyWith<_$SdkError_InvalidUrlImpl> get copyWith =>
      __$$SdkError_InvalidUrlImplCopyWithImpl<_$SdkError_InvalidUrlImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return invalidUrl(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return invalidUrl?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (invalidUrl != null) {
      return invalidUrl(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return invalidUrl(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return invalidUrl?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (invalidUrl != null) {
      return invalidUrl(this);
    }
    return orElse();
  }
}

abstract class SdkError_InvalidUrl extends SdkError {
  const factory SdkError_InvalidUrl({required final String msg}) =
      _$SdkError_InvalidUrlImpl;
  const SdkError_InvalidUrl._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_InvalidUrlImplCopyWith<_$SdkError_InvalidUrlImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_DbImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_DbImplCopyWith(
          _$SdkError_DbImpl value, $Res Function(_$SdkError_DbImpl) then) =
      __$$SdkError_DbImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_DbImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_DbImpl>
    implements _$$SdkError_DbImplCopyWith<$Res> {
  __$$SdkError_DbImplCopyWithImpl(
      _$SdkError_DbImpl _value, $Res Function(_$SdkError_DbImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_DbImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_DbImpl extends SdkError_Db {
  const _$SdkError_DbImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.db(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_DbImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_DbImplCopyWith<_$SdkError_DbImpl> get copyWith =>
      __$$SdkError_DbImplCopyWithImpl<_$SdkError_DbImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return db(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return db?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (db != null) {
      return db(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return db(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return db?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (db != null) {
      return db(this);
    }
    return orElse();
  }
}

abstract class SdkError_Db extends SdkError {
  const factory SdkError_Db({required final String msg}) = _$SdkError_DbImpl;
  const SdkError_Db._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_DbImplCopyWith<_$SdkError_DbImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_NoAccountImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_NoAccountImplCopyWith(_$SdkError_NoAccountImpl value,
          $Res Function(_$SdkError_NoAccountImpl) then) =
      __$$SdkError_NoAccountImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_NoAccountImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_NoAccountImpl>
    implements _$$SdkError_NoAccountImplCopyWith<$Res> {
  __$$SdkError_NoAccountImplCopyWithImpl(_$SdkError_NoAccountImpl _value,
      $Res Function(_$SdkError_NoAccountImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_NoAccountImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_NoAccountImpl extends SdkError_NoAccount {
  const _$SdkError_NoAccountImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.noAccount(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_NoAccountImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_NoAccountImplCopyWith<_$SdkError_NoAccountImpl> get copyWith =>
      __$$SdkError_NoAccountImplCopyWithImpl<_$SdkError_NoAccountImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return noAccount(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return noAccount?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (noAccount != null) {
      return noAccount(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return noAccount(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return noAccount?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (noAccount != null) {
      return noAccount(this);
    }
    return orElse();
  }
}

abstract class SdkError_NoAccount extends SdkError {
  const factory SdkError_NoAccount({required final String msg}) =
      _$SdkError_NoAccountImpl;
  const SdkError_NoAccount._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_NoAccountImplCopyWith<_$SdkError_NoAccountImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SdkError_CancelledImplCopyWith<$Res>
    implements $SdkErrorCopyWith<$Res> {
  factory _$$SdkError_CancelledImplCopyWith(_$SdkError_CancelledImpl value,
          $Res Function(_$SdkError_CancelledImpl) then) =
      __$$SdkError_CancelledImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String msg});
}

/// @nodoc
class __$$SdkError_CancelledImplCopyWithImpl<$Res>
    extends _$SdkErrorCopyWithImpl<$Res, _$SdkError_CancelledImpl>
    implements _$$SdkError_CancelledImplCopyWith<$Res> {
  __$$SdkError_CancelledImplCopyWithImpl(_$SdkError_CancelledImpl _value,
      $Res Function(_$SdkError_CancelledImpl) _then)
      : super(_value, _then);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? msg = null,
  }) {
    return _then(_$SdkError_CancelledImpl(
      msg: null == msg
          ? _value.msg
          : msg // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SdkError_CancelledImpl extends SdkError_Cancelled {
  const _$SdkError_CancelledImpl({required this.msg}) : super._();

  @override
  final String msg;

  @override
  String toString() {
    return 'SdkError.cancelled(msg: $msg)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SdkError_CancelledImpl &&
            (identical(other.msg, msg) || other.msg == msg));
  }

  @override
  int get hashCode => Object.hash(runtimeType, msg);

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SdkError_CancelledImplCopyWith<_$SdkError_CancelledImpl> get copyWith =>
      __$$SdkError_CancelledImplCopyWithImpl<_$SdkError_CancelledImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(int status, String msg) urlExpired,
    required TResult Function(String msg) notFound,
    required TResult Function(int status, String msg) http,
    required TResult Function(String msg) server,
    required TResult Function(String msg) source,
    required TResult Function(String msg) network,
    required TResult Function(String msg) io,
    required TResult Function(String msg) md5Mismatch,
    required TResult Function(String msg) invalidUrl,
    required TResult Function(String msg) db,
    required TResult Function(String msg) noAccount,
    required TResult Function(String msg) cancelled,
  }) {
    return cancelled(msg);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(int status, String msg)? urlExpired,
    TResult? Function(String msg)? notFound,
    TResult? Function(int status, String msg)? http,
    TResult? Function(String msg)? server,
    TResult? Function(String msg)? source,
    TResult? Function(String msg)? network,
    TResult? Function(String msg)? io,
    TResult? Function(String msg)? md5Mismatch,
    TResult? Function(String msg)? invalidUrl,
    TResult? Function(String msg)? db,
    TResult? Function(String msg)? noAccount,
    TResult? Function(String msg)? cancelled,
  }) {
    return cancelled?.call(msg);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(int status, String msg)? urlExpired,
    TResult Function(String msg)? notFound,
    TResult Function(int status, String msg)? http,
    TResult Function(String msg)? server,
    TResult Function(String msg)? source,
    TResult Function(String msg)? network,
    TResult Function(String msg)? io,
    TResult Function(String msg)? md5Mismatch,
    TResult Function(String msg)? invalidUrl,
    TResult Function(String msg)? db,
    TResult Function(String msg)? noAccount,
    TResult Function(String msg)? cancelled,
    required TResult orElse(),
  }) {
    if (cancelled != null) {
      return cancelled(msg);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SdkError_UrlExpired value) urlExpired,
    required TResult Function(SdkError_NotFound value) notFound,
    required TResult Function(SdkError_Http value) http,
    required TResult Function(SdkError_Server value) server,
    required TResult Function(SdkError_Source value) source,
    required TResult Function(SdkError_Network value) network,
    required TResult Function(SdkError_Io value) io,
    required TResult Function(SdkError_Md5Mismatch value) md5Mismatch,
    required TResult Function(SdkError_InvalidUrl value) invalidUrl,
    required TResult Function(SdkError_Db value) db,
    required TResult Function(SdkError_NoAccount value) noAccount,
    required TResult Function(SdkError_Cancelled value) cancelled,
  }) {
    return cancelled(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SdkError_UrlExpired value)? urlExpired,
    TResult? Function(SdkError_NotFound value)? notFound,
    TResult? Function(SdkError_Http value)? http,
    TResult? Function(SdkError_Server value)? server,
    TResult? Function(SdkError_Source value)? source,
    TResult? Function(SdkError_Network value)? network,
    TResult? Function(SdkError_Io value)? io,
    TResult? Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult? Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult? Function(SdkError_Db value)? db,
    TResult? Function(SdkError_NoAccount value)? noAccount,
    TResult? Function(SdkError_Cancelled value)? cancelled,
  }) {
    return cancelled?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SdkError_UrlExpired value)? urlExpired,
    TResult Function(SdkError_NotFound value)? notFound,
    TResult Function(SdkError_Http value)? http,
    TResult Function(SdkError_Server value)? server,
    TResult Function(SdkError_Source value)? source,
    TResult Function(SdkError_Network value)? network,
    TResult Function(SdkError_Io value)? io,
    TResult Function(SdkError_Md5Mismatch value)? md5Mismatch,
    TResult Function(SdkError_InvalidUrl value)? invalidUrl,
    TResult Function(SdkError_Db value)? db,
    TResult Function(SdkError_NoAccount value)? noAccount,
    TResult Function(SdkError_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    if (cancelled != null) {
      return cancelled(this);
    }
    return orElse();
  }
}

abstract class SdkError_Cancelled extends SdkError {
  const factory SdkError_Cancelled({required final String msg}) =
      _$SdkError_CancelledImpl;
  const SdkError_Cancelled._() : super._();

  @override
  String get msg;

  /// Create a copy of SdkError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SdkError_CancelledImplCopyWith<_$SdkError_CancelledImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
