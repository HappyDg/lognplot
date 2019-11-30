""" Time series database.
"""

from .series import ZoomSerie
from .aggregation import Aggregation
from ..time import TimeSpan


class TsDb:
    """ A time series database.
    """

    def __init__(self):
        # TODO: load / store data in file!
        self._traces = {}  # The internal trace data.

    def signal_names(self):
        """ Get a sorted list of signal names. """
        return list(sorted(self._traces))

    def get_or_create_serie(self, name):
        if name in self._traces:
            serie = self._traces[name]
        else:
            serie = ZoomSerie()
            self._traces[name] = serie
        return serie

    def add_sample(self, name: str, sample):
        """ Add a single sample to the given series. """
        serie = self.get_or_create_serie(name)
        serie.add_sample(sample)

    def add_samples(self, name: str, samples):
        """ Add samples to the given series. """
        serie = self.get_or_create_serie(name)
        serie.add_samples(samples)

    def query_len(self, name: str) -> int:
        """ Get the length of a given series. """
        serie = self.get_or_create_serie(name)
        return len(serie)

    def query_summary(self, name: str, timespan=None) -> Aggregation:
        serie = self.get_or_create_serie(name)
        return serie.query_summary(selection_timespan=timespan)

    def query(self, name: str, timespan: TimeSpan, count: int):
        """ Query the database on the given signal.
        """
        serie = self.get_or_create_serie(name)
        return serie.query(timespan, count)
